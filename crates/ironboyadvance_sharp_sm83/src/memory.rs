pub trait MemoryInterface {
    fn load_8(&self, address: u16) -> u8;

    fn load_16(&self, address: u16) -> u16;

    fn store_8(&mut self, address: u16, value: u8);

    fn store_16(&mut self, address: u16, value: u16);

    fn idle_cycle(&mut self);

    fn change_speed(&mut self);
}
