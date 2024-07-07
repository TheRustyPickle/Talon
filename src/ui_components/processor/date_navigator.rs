use chrono::{Days, Months};

use crate::ui_components::processor::{DatePickerHandler, NavigationType};

#[derive(Default)]
pub struct DateNavigator {
    nav_type: NavigationType,
    handler: DatePickerHandler,
}

impl DateNavigator {
    pub fn handler(&mut self) -> &mut DatePickerHandler {
        &mut self.handler
    }

    pub fn nav_type(&mut self) -> &mut NavigationType {
        &mut self.nav_type
    }

    pub fn nav_name(&self) -> String {
        self.nav_type.to_string()
    }

    pub fn go_next(&mut self) {
        match self.nav_type {
            NavigationType::Day => self.next_day(),
            NavigationType::Week => self.next_week(),
            NavigationType::Month => self.next_month(),
            NavigationType::Year => self.next_year(),
        }
    }

    pub fn go_previous(&mut self) {
        match self.nav_type {
            NavigationType::Day => self.previous_day(),
            NavigationType::Week => self.previous_week(),
            NavigationType::Month => self.previous_month(),
            NavigationType::Year => self.previous_year(),
        }
    }

    fn next_day(&mut self) {
        let from_date = self.handler().from;
        let mut to_date = self.handler().to;

        if from_date != to_date {
            *self.handler().from() = to_date;
            return;
        }

        to_date = to_date.checked_add_days(Days::new(1)).unwrap();

        *self.handler().from() = to_date;
        *self.handler().to() = to_date;
    }

    fn previous_day(&mut self) {
        let mut from_date = self.handler().from;
        let to_date = self.handler().to;

        if from_date != to_date {
            *self.handler().to() = from_date;
            return;
        }

        from_date = from_date.checked_sub_days(Days::new(1)).unwrap();

        *self.handler().from() = from_date;
        *self.handler().to() = from_date;
    }

    fn next_week(&mut self) {
        let from_date = self.handler().from;
        let mut to_date = self.handler().to;

        let target_date = to_date.checked_sub_days(Days::new(7)).unwrap();

        if from_date != target_date {
            *self.handler().from() = target_date;
            return;
        }

        to_date = to_date.checked_add_days(Days::new(7)).unwrap();

        *self.handler().from() = to_date.checked_sub_days(Days::new(7)).unwrap();
        *self.handler().to() = to_date;
    }

    fn previous_week(&mut self) {
        let mut from_date = self.handler().from;
        let to_date = self.handler().to;

        let target_date = from_date.checked_add_days(Days::new(7)).unwrap();

        if to_date != target_date {
            *self.handler().to() = target_date;
            return;
        }

        from_date = from_date.checked_sub_days(Days::new(7)).unwrap();

        *self.handler().from() = from_date;
        *self.handler().to() = from_date.checked_add_days(Days::new(7)).unwrap();
    }

    fn next_month(&mut self) {
        let from_date = self.handler().from;
        let mut to_date = self.handler().to;

        let target_date = to_date.checked_sub_months(Months::new(1)).unwrap();

        if from_date != target_date {
            *self.handler().from() = target_date;
            return;
        }

        to_date = to_date.checked_add_months(Months::new(1)).unwrap();

        *self.handler().from() = to_date.checked_sub_months(Months::new(1)).unwrap();
        *self.handler().to() = to_date;
    }

    fn previous_month(&mut self) {
        let mut from_date = self.handler().from;
        let to_date = self.handler().to;

        let target_date = from_date.checked_add_months(Months::new(1)).unwrap();

        if to_date != target_date {
            *self.handler().to() = target_date;
            return;
        }

        from_date = from_date.checked_sub_months(Months::new(1)).unwrap();

        *self.handler().from() = from_date;
        *self.handler().to() = from_date.checked_add_months(Months::new(1)).unwrap();
    }

    fn next_year(&mut self) {
        let from_date = self.handler().from;
        let mut to_date = self.handler().to;

        let target_date = to_date.checked_sub_months(Months::new(12)).unwrap();

        if from_date != target_date {
            *self.handler().from() = target_date;
            return;
        }

        to_date = to_date.checked_add_months(Months::new(12)).unwrap();

        *self.handler().from() = to_date.checked_sub_months(Months::new(12)).unwrap();
        *self.handler().to() = to_date;
    }

    fn previous_year(&mut self) {
        let mut from_date = self.handler().from;
        let to_date = self.handler().to;

        let target_date = from_date.checked_add_months(Months::new(12)).unwrap();

        if to_date != target_date {
            *self.handler().to() = target_date;
            return;
        }

        from_date = from_date.checked_sub_months(Months::new(12)).unwrap();

        *self.handler().from() = from_date;
        *self.handler().to() = from_date.checked_add_months(Months::new(12)).unwrap();
    }
}
