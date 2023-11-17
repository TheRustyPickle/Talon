<div align="center"><h1>Talon</h1></div>
<div align="center">
<a href="https://wakatime.com/badge/github/TheRustyPickle/Talon"><img src="https://wakatime.com/badge/github/TheRustyPickle/Talon.svg" alt="wakatime"></a>
</div>

Talon is a tool to generate on-demand data insights from public Telegram chats. Powered by Rust, grammers and egui, it offers a straightforward interface that leverages the Telegram account API.

## Features

### 1. Counter

- Takes an input of a starting point Telegram message link (and an optional ending point).
- Displays the total number of unique users found and the total messages counted.
- Utilizes gathered data to visualize additional analytics.

### 2. User Table

- Utilizes the counted data to generate a comprehensive user table.
- View user details, including name, username, ID, total messages, total words, total characters, and more.
- Allows interaction with the table, such as selecting cells and copying data in an organized manner.

### 3. Session Creation

- Takes relevant input to log in to a Telegram account and create a new session.
- Choose between a temporary session (logs out on app close) or a non-temporary session (creates a file for persistent login).

## Work In Progress

- **Whitelisting**: Allow users to highlight and separate data for specific whitelisted users.
- **Charts**: Visualize chat data with different charts, including message activity.

## Important Note

Talon uses the [grammers library](https://github.com/lonami/grammers) for Telegram operations. Please be aware that grammers is currently under development and may not be stable or audited for security.

## Installation

**1. Run from Source Code:**

```bash
git clone https://github.com/TheRustyPickle/Talon
cd Talon
cargo run --release
```

**2. Run the Latest Release:**

To be added

**3. Install using Cargo:**

To be added

## Feedback and Contributions

Have feedback, found a bug, or have a feature request? Feel free to [open an issue](https://github.com/TheRustyPickle/Talon/issues/new). Pull requests are welcome!

## License

Talon is under the [MIT License](LICENSE).
