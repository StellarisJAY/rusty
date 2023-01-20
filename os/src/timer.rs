use riscv::register::time;
use crate::sbi::set_timer;
use crate::config::TIME_FREQUENCY;
const TICK_PER_SECOND: usize = 100;

pub fn get_time() -> usize {
    time::read()
}

pub fn set_next_time_trigger() {
    set_timer(get_time() + TIME_FREQUENCY / TICK_PER_SECOND);
}