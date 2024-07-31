#[derive(Default, Clone)]
pub struct CounterCounts {
    pub total_message: i32,
    pub whitelisted_message: i32,
    pub total_user: i32,
    pub whitelisted_user: i32,
    pub deleted_message: i32,
}

impl CounterCounts {
    pub fn set_total_user(&mut self, total_user: i32) {
        self.total_user = total_user;
    }

    pub fn add_one_total_message(&mut self) {
        self.total_message += 1;
    }

    pub fn add_deleted_message(&mut self, to_add: i32) {
        if to_add > 0 {
            self.deleted_message += to_add;
        }
    }
}
