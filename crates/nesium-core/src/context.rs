use crate::config::region::Region;

#[derive(Debug)]
pub enum Context<'a> {
    None,
    Some { region: &'a mut Region },
}
