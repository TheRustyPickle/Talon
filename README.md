<div align="center"><h1>Talon</h1></div>
<div align="center">
<a href="https://wakatime.com/badge/github/TheRustyPickle/Talon"><img src="https://wakatime.com/badge/github/TheRustyPickle/Talon.svg" alt="wakatime"></a>
</div>

Talon is a tool to generate on-demand data insights from public Telegram chats. Powered by Rust, grammers, and egui, it offers a straightforward interface that leverages the Telegram account API.

![Screenshot](https://github.com/TheRustyPickle/Talon/assets/35862475/68b5f14f-d717-4911-b42d-9f15088a48ac)

## Features

**Counter:**

- Checks Telegram messages from a given message link (and an optional ending point).
- Able to utilize multiple sessions, dramatically increasing checking speed. Tested with up to 8 sessions and 300k messages
- Displays the number of unique users found, the total messages counted, and others.
- Utilizes gathered data to visualize additional analytics.

**User Table:**

- Utilizes the counted data to generate a comprehensive user table.
- View user details, including name, username, ID, total messages, total words, total characters, and more.
- Allows interaction with the table, such as selecting cells and copying data in an organized manner.

**Session Creation:**

- Takes relevant input to log in to a Telegram account and create a new session.
- Choose between a temporary session (logs out on app close) or a non-temporary session (creates a file for persistent login).

**Whitelist:**

- Allows grouping specific users and enabling viewing their data separately.
- Easily add or remove users from the whitelist as necessary.

**Charts:**

- Visualize message counts or active users on an hourly, daily, weekly, or monthly basis.
- Explore chat activity based on the day of the week.

## Important Note

Talon uses the [grammers library](https://github.com/lonami/grammers) for Telegram operations. Please be aware that grammers is currently under development and may not be stable or audited for security.

## Installation

**1. Run from Source Code:**

- Clone the repository `git clone https://github.com/TheRustyPickle/Talon`
- Run with Cargo `cargo run --release`

**2. Run the latest Release:**

To be added

**3. Install using Cargo:**

To be added

## App Data Location

See [here](https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html) for location info where app data is saved which is determined based on the OS. Files can be added, deleted, or modified here to reflect in the application.

## Feedback and Contributions

Have feedback, found a bug, or have a feature request? Feel free to [open an issue](https://github.com/TheRustyPickle/Talon/issues/new). Pull requests are welcome!

## License

Talon is under the [MIT License](LICENSE).
