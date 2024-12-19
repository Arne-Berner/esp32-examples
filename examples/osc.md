# setup
Before you run the access point you need to export the export-example.sh file
```sh
export export-example.sh
# for fish
. export-example.sh
```
To run the access point use
```sh
cargo run --example osc --release
```
Then connect to the AP using the credentials in the bash script and use something like oscd to send messages to the server on the exact ip that the AP has or to 0.0.0.0 using port 9000

