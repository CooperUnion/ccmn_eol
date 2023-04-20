use ccmn_eol_shared::gpiotest::EolGpios;

pub fn do_gpio_test() {
    let gpios = EolGpios::new();
    gpios.init();

    gpios.set_all_to_output();
    gpios.write_all(u64::MAX);
}
