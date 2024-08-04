<div align="center"><h1>Talon</h1></div>
<div align="center">
<a href="https://wakatime.com/badge/github/TheRustyPickle/Talon"><img src="https://wakatime.com/badge/github/TheRustyPickle/Talon.svg" alt="wakatime"></a>
<a href="https://crates.io/crates/talon-gui"><img src="https://img.shields.io/crates/v/talon-gui.svg?style=flat-square&logo=rust&color=orange" alt="Crates version"/></a>
<a href="https://github.com/TheRustyPickle/Talon/releases/latest"><img src="https://img.shields.io/github/v/release/TheRustyPickle/Talon?style=flat-square&logo=github&color=orange" alt="Release Version"/></a>
<a href="https://crates.io/crates/talon-gui"><img src="https://img.shields.io/crates/d/talon-gui?style=flat-square" alt="Downloads"/></a>
</div>

Talon is a tool to generate on-demand data insights from public Telegram chats. Powered by Rust, grammers, and egui, it offers a straightforward interface that leverages the Telegram account API.

![Screenshot](https://github.com/TheRustyPickle/Talon/assets/35862475/56b834d8-992b-413c-81e7-aaf023e00047)

## Features

- **User and Message Metrics:** Displays the number of unique users, total messages counted, and other info.
- **Detailed User Insights:** View comprehensive user details including name, username, ID, total messages, total words, total characters, and more.
- **Interactive Data Table:** Select cells, interact with the table and allows copying cells in an organized manner.
- **Visual Analytics:** Visualize message counts and active users on an hourly, daily, weekly, monthly and by the day of the week basis.
- **Date Range and Navigation:** Easily navigate and view table and chart data within a specific date range with buttons to cycle by day, week, month, or year.
- **Session Management:** Options to choose between temporary sessions (logs out on app close) or non-temporary sessions (creates a file for persistent login).
- **User Grouping:** Group specific users by whitelisting to view their data separately and analyze their activity.
- **Blacklisting:**: Exclude specific users from data analysis to prevent their data from appearing in the results.
- **Multi-Session Capability:** Utilize multiple sessions to dramatically increase checking speed, tested with up to 12 sessions and 300k messages.
- **Multi-Chat Capability:** Analyze multiple chats simultaneously and view data from each chat separately.

## Important Note

Talon uses the [grammers library](https://github.com/lonami/grammers) for Telegram operations. Please be aware that grammers is currently under development and may not be stable or audited for security.

## Installation

**1. Run from Source Code:**

- Clone the repository `git clone https://github.com/TheRustyPickle/Talon`
- Run with Cargo `cargo run --release`

**2. Run the latest Release:**

- Download the latest executable from [Releases](https://github.com/TheRustyPickle/Talon/releases/latest).
- Unzip the executable and double click to start the app.

**3. Install using Cargo:**

- Install using `cargo install talon-gui`
- Start with `talon`

## App Data Location

See [here](https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html) for location info where app data is saved which is determined based on the OS. Files can be added, deleted, or modified here to reflect in the application.

## Disclaimer

Talon is designed to work only with public Telegram chats and does not support private groups. The app operates entirely on-demand, without saving any data or analyzing messages beyond counting them and checking the timestamp. Users are responsible for ensuring their use of this tool complies with Telegram’s Terms of Service and relevant laws.

## Feedback and Contributions

Have feedback, found a bug, or have a feature request? Feel free to [open an issue](https://github.com/TheRustyPickle/Talon/issues/new). Pull requests are welcome!

## License

Talon is under the [MIT License](LICENSE).
