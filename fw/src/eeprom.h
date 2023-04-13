#ifndef EEPROM_H
#define EEPROM_H

#include <stddef.h>
#include <stdint.h>

void eeprom_init(void);

int eeprom_read(uint16_t addr, uint8_t *data, size_t len);
int eeprom_write(uint16_t addr, const uint8_t *data, size_t len);
int eeprom_write_byte(uint16_t addr, uint8_t data);

#endif
