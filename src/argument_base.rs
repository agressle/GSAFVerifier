#[derive(Clone)]
///The arguments of the instance.
pub struct ArgumentBase {

    ///The ID of the argument. The same that is used in the input file.
    id: usize,
    ///The IDs of the attacks of the original instance that attack this argument. Ordered ascending.
    attacked_by: Vec<usize>
}

impl ArgumentBase {

    pub fn new<'a>() -> ArgumentBase {
        ArgumentBase {
            id: 0,
            attacked_by: Vec::new()
        }
    }

    #[inline]
    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    #[inline]
    pub fn get_id(&self) -> usize {
        self.id
    }

    #[inline]
    pub fn get_attacked_by(&self) -> &Vec<usize> {
        &self.attacked_by
    }

    #[inline]
    pub fn add_attacked_by(&mut self, attack_index: usize) {
        self.attacked_by.push(attack_index);
    }


}