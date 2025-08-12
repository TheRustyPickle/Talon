use chrono::NaiveDate;

/// Handler for the `DatePicker` in the UI for data changes
#[derive(Default)]
pub struct DatePickerHandler {
    /// The From Date currently selected in the UI
    pub from: NaiveDate,
    /// The To Date currently selected in the UI
    pub to: NaiveDate,
    /// The last From date selected before the current From date
    last_from: Option<NaiveDate>,
    /// The last To date selected before the current To date
    last_to: Option<NaiveDate>,
    /// The oldest date with at least 1 data point
    start: Option<NaiveDate>,
    /// The newest date with at least 1 data point
    end: Option<NaiveDate>,
}

impl DatePickerHandler {
    pub fn from(&mut self) -> &mut NaiveDate {
        &mut self.from
    }
    pub fn to(&mut self) -> &mut NaiveDate {
        &mut self.to
    }
    /// Verify whether the current From and To dates have changed
    pub fn check_date_change(&mut self) -> bool {
        if let Some(d) = self.last_from
            && d != self.from
        {
            if self.from > self.to {
                self.from = self.to;
            }

            self.last_from = Some(self.from);
            return true;
        }
        if let Some(d) = self.last_to
            && d != self.to
        {
            if self.to < self.from {
                self.to = self.from;
            }

            self.last_to = Some(self.to);
            return true;
        }
        false
    }

    /// Reset dates to the oldest and the newest value
    pub fn reset_dates(&mut self) {
        self.from = self.start.unwrap();
        self.to = self.end.unwrap();
        self.last_from = Some(self.from);
        self.last_to = Some(self.to);
    }

    /// Compare the given date with the current Start and End date
    /// to find the oldest and the newest date
    pub fn update_dates(&mut self, date: NaiveDate) {
        if self.start.is_none_or(|current| current > date) {
            self.from = date;
            self.start = Some(date);
            self.last_from = Some(date);
        }

        if self.end.is_none_or(|current_date| current_date < date) {
            self.to = date;
            self.end = Some(date);
            self.last_to = Some(date);
        }
    }

    /// Whether the given date is whtin the current From and To range
    pub fn within_range(&self, date: NaiveDate) -> bool {
        date >= self.from && date <= self.to
    }

    /// Whether the given date is before the current To range
    pub fn before_to_range(&self, date: NaiveDate) -> bool {
        date < self.to
    }
}
