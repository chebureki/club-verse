use crate::pkg::xt::IntoXT;

pub struct PlayerMove{
    x: usize,
    y: usize,

}

impl IntoXT for PlayerMove{
    fn into_xt<'raw>(&'raw self) -> super::XTPackage<'raw> {
        
    }
}



// pub struct PlayerMoveResp{
//
// }
