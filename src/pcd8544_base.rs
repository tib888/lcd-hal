pub trait Pcd8544Base {
    fn command(&mut self, u8);
    fn data(&mut self, &[u8]);
}
