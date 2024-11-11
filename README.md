> [!NOTE]
> This project is sponsered by [proxies.fo](https://proxies.fo/), check them out for the best proxies on the market.

# Grass Node
Automate [Grass](https://getgrass.io/) Desktop Node with [proxies](https://proxies.fo).

## Features
- Identified as Desktop Node (2X points).
- Works with HTTP, Socks5, Socks5h, Socks4 proxies.
- Blazing fast (We ❤ Rust).

## Setup
1. Visit [Grass](https://getgrass.io/) and register an account.
2. Press F12 on the Dashboard page and go to the Console tab.
3. Paste the following script into the console and copy the value.
    ```js 
    const userId = localStorage.getItem('userId');

    if (userId) {
        console.log('Your user ID:', userId);
    } else {
        console.log('Failed to get userId');
    }
   ```
4. Your user ID should look like this: `xxxxxxxxxxxxxxxxxxxxxxxxxxx`. Do not copy the quotes.

## Usage
1. Install Rust from [here](https://www.rust-lang.org/tools/install).
2. Clone the repository.
    ```sh
    git clone https://github.com/twine-solutions/grass-node.git
    cd grass-node
    ```
3. Run the project.
    ```sh
    cargo run --release -- --user-id <USER_ID> --proxies <PROXY_FILE>
    ```
4. Replace `<USER_ID>` with the user ID you copied earlier.
5. Replace `<PROXY_FILE>` with the path to your proxies file. The file should contain one proxy per line in the format `protocol://user:pass@ip:port`.

# Credits
- [proxies.fo](https://proxies.fo) for being a great proxy provider.
- [Solana0x](https://github.com/Solana0x/GrassNode2/) for their Python implementation.