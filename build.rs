fn main() {
    #[cfg(windows)]
    {
        embed_resource::compile("res/PopMax.rc", embed_resource::NONE);
    }
}
