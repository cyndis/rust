#[crate_type = "lib"];

pub struct Fish {
    x: int
}

mod unexported {
    use super::Fish;
    impl Fish : Eq {
        pure fn eq(&self, _: &Fish) -> bool { true }
        pure fn ne(&self, _: &Fish) -> bool { false }
    }
}


