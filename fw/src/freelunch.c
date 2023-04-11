#include <stdint.h>
#include <stdio.h>

#include <driver/twai.h>
#include <esp_log.h>
#include <freertos/FreeRTOS.h>
#include <freertos/ringbuf.h>
#include <freertos/task.h>
#include <tinyusb.h>
#include <tusb.h>
#include <tusb_cdc_acm.h>
#include <tusb_console.h>
#include <sdkconfig.h>

static const char *TAG = "freelunch";

static RingbufHandle_t canrx_ringbuf;

static void freelunch_send(uint32_t id, const uint8_t *data, uint8_t len);

#define FREELUNCH_ITF TINYUSB_CDC_ACM_0
#define CONSOLE_ITF TINYUSB_CDC_ACM_1

_Atomic uint32_t freelunch_messages_sent = 0;
_Atomic bool freelunch_channel_connected_and_ready = false;

void freelunch_can_rx_callback(const twai_message_t* message) {
    BaseType_t res = xRingbufferSend(canrx_ringbuf, message, sizeof(*message), pdMS_TO_TICKS(0));
    if (res != pdTRUE) {
        ESP_LOGE(TAG, "Failed to send message to ringbuf");
    }
}

static void freelunch_send(uint32_t id, const uint8_t *data, uint8_t len)
{
    uint8_t buf[32] = {0};

    static uint8_t hex[17] = "0123456789ABCDEF";

    buf[0] = 'T';
    buf[1] = hex[(id >> 28) & 0x1];
    buf[2] = hex[(id >> 24) & 0xF];
    buf[3] = hex[(id >> 20) & 0xF];
    buf[4] = hex[(id >> 16) & 0xF];
    buf[5] = hex[(id >> 12) & 0xF];
    buf[6] = hex[(id >>  8) & 0xF];
    buf[7] = hex[(id >>  4) & 0xF];
    buf[8] = hex[(id >>  0) & 0xF];

    buf[9] = len + 48; // convert to ASCII

    int pos = 9;

    for (int i = 0; i < len; i++) {
        buf[++pos] = hex[data[i] >> 4];
        buf[++pos] = hex[data[i] & 0x0F];
    }

    buf[++pos] = '\r';

    // write out data to USB
    const uint8_t *out = buf;
    size_t out_len = pos + 1;

    // strategy from https://github.com/micropython/micropython/pull/7943/files
    while (out_len > 0 && freelunch_channel_connected_and_ready) {
        const size_t l = tinyusb_cdcacm_write_queue(0, out, out_len);
        out += l;
        out_len -= l;
        tud_cdc_n_write_flush(0);
    }

    freelunch_messages_sent++;
}

static void freelunch_canrx_task()
{
    for (;;) {
        size_t item_size;
        const twai_message_t *msg = (const twai_message_t*)xRingbufferReceive(canrx_ringbuf, &item_size, portMAX_DELAY);

        if (!msg) continue;
        if (item_size == 0) continue;

        freelunch_send(msg->identifier, msg->data, msg->data_length_code);

        vRingbufferReturnItem(canrx_ringbuf, (void *)msg);
    }
}

static void usb_callback_freelunch_line_state_changed(int itf, cdcacm_event_t *event)
{
    const bool dtr = event->line_state_changed_data.dtr;
    const bool rts = event->line_state_changed_data.rts;

    freelunch_channel_connected_and_ready = dtr && rts;
}

void freelunch_init(void)
{
    ESP_LOGI(TAG, "USB initialization");
    tinyusb_config_t tusb_cfg = {0}; // the configuration using default values
    ESP_ERROR_CHECK(tinyusb_driver_install(&tusb_cfg));

    const tinyusb_config_cdcacm_t acm_cfg = {
        .usb_dev = TINYUSB_USBDEV_0,
        .cdc_port = FREELUNCH_ITF,
        .rx_unread_buf_sz = 64,
        .callback_line_state_changed = usb_callback_freelunch_line_state_changed,
    };

    ESP_ERROR_CHECK(tusb_cdc_acm_init(&acm_cfg));

    const tinyusb_config_cdcacm_t acm_cfg2 = {
        .usb_dev = TINYUSB_USBDEV_0,
        .cdc_port = CONSOLE_ITF,
        .rx_unread_buf_sz = 64,
    };

    ESP_ERROR_CHECK(tusb_cdc_acm_init(&acm_cfg2));
    const esp_err_t err = esp_tusb_init_console(CONSOLE_ITF);
    // if (err != ESP_OK) {
    //     esp_tusb_deinit_console(CONSOLE_ITF);
    //     fprintf(stderr, "Failed to initialize console on cdc: %s\n", esp_err_to_name(err));
    // }

    ESP_LOGI(TAG, "USB initialization DONE");

    // initialize ringbuffer
    canrx_ringbuf = xRingbufferCreate(2048, RINGBUF_TYPE_NOSPLIT);
    if (canrx_ringbuf == NULL) {
        ESP_LOGE(TAG, "Failed to create rx ring buffer.");
    }

    // create tx task
    static TaskHandle_t freelunch_canrx_task_handle;
    xTaskCreatePinnedToCore(freelunch_canrx_task, "freelunch", 4096, 0, 2, &freelunch_canrx_task_handle, 1);
}
