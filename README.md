This project is a solution to an assignment.
It contains a single executable.

The program in question has two modes of operation.
When invoked without arguments it listens for incoming connections on a dynamically allocated port on the localhost address (127.0.0.1).

Upon receiving a GET request on `/pair` it generates an "authentication token" (a short pseudorandom string) and sends it in the response.
The token is also appended to a JSON array in a file named `tokens.json`.

Upon receiving a POST request on `/messages` the program tries to deserialize an event record.
An event consists of an `uuid` field (a string) and a `msg` (a wrapper around a string).
It then proceeds to match the provided `uuid` token against tokens stored in `tokens.json`.
If no matching token is found a 401 error is returned.
Otherwise the message is printed to stdout and appended to an array of JSON objects stored in a file named `messages.json`.

When invoked with `-H` and `-m` flags, the program attempts to obtain a token from another instance of the executable listening at the address specified with the `-H` flag.
Upon success it uses the token to send the message following the `-m` flag.

## Example usage and testing

Build the executable with

```
cargo build
```

and run the first instance as follows

```
> cargo run
Listening on http://127.0.0.1:65126
```

In another window execute

```
cargo run -- -H 127.0.0.1:65126 -m "some text"
```

The string `some text` should appear in the first window.
Files `tokens.json` with the token and `messages.json` with the message should appear in the first program instance's working directory.

An example showing authentication attempts using valid and invalid tokens.

```
> curl -i 127.0.0.1:65126/pair | tail -1
ydXnoMD

> curl -i 127.0.0.1:65126/pair | tail -1
CCusUa9

> cat tokens.json
["ydXnoMD","CCusUa9"]

> curl -i 127.0.0.1:65126/messages -XPOST -d "{\"msg\": {\"payload\": \"good\"}, \"uuid\": \"CCusUa9\"}" | head -1
HTTP/1.1 200 OK

> curl -i 127.0.0.1:65126/messages -XPOST -d "{\"msg\": {\"payload\": \"bad\"}, \"uuid\": \"INVALID\"}" | head -1
HTTP/1.1 401 Unauthorized

> cat messages.json
[{"payload":"good"}]
```
