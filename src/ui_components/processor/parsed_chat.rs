#[derive(Default, Clone)]
pub struct ParsedChat {
    name: String,
    start_point: Option<i32>,
    end_point: Option<i32>,
}

impl ParsedChat {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn start_point(&self) -> Option<i32> {
        self.start_point
    }

    pub fn end_point(&self) -> Option<i32> {
        self.end_point
    }

    pub fn new(name: String, start_point: Option<i32>, end_point: Option<i32>) -> Self {
        Self {
            name,
            start_point,
            end_point,
        }
    }

    pub fn set_end_point(&mut self, point: i32) -> bool {
        if let Some(start) = self.start_point
            && point >= start
        {
            return false;
        }
        self.end_point = Some(point);
        true
    }
}
