# Token Server
Server to provide one-time access tokens for some set of meta data

The purpose of this server is to provide access tokens, for instance for 
web or REST servers.

The token server should run in a protected environment, that can only be accessed
from the local network.
A web server collects some metadata which will be used in all subsequent
access to the web server. The web server requests a new token from
the token server and includes that in the response to the user. On the next
request from the user to the web server, the web server retrieves the token
from the request and sends an update request to the token server. If that succeeds,
the web server can include the metadata into the repsonse, as well as the new token.
This process repeats for each subsequent user request.
In case the token server responds with an Invalid Token response, the web server
might decide to redirect the user to the login page.

If there is more state to keep track of e.g., the year and month 
or a business unit to work with, the web server can add these values as metadata in
the update request to the token server. The token server will add or update the key/value
pairs into the existing metadata for the token.  

DISCLAIMER: Not suited for high-volume metadata sets. My estimate is that it will scale
            well enough for up to 100,000 pieces of metadata, and up to 
            1,000 transactions per second.
            To store more data, or allow for more traffic, a database backend seems
            more appropriate

## Usage
Usage: RUST_LOG='tower_http=trace,token_server=debug' cargo run [OPTIONS]

Optional arguments:
  -h, --help       print this help message
  -d, --dump       allow for HEAD /dump endpoint to log all metadata
  -p, --port PORT  Which port to listen on (default: 3666)
  -P, --purge-interval PURGE-INTERVAL
                   What frequency to remove expired tokens, between 1s and 90min (default: 1min)
  -t, --token-lifetime TOKEN-LIFETIME
                   How long does a token remain valid, between 30min and 96h (default: 2h)

## REST API

  * POST /token
        Create a new token for the provided metadata in the request body

        Returns: (text/plain) the new token


  * PUT /token
        Exchange token for a new one
        Optionally add metadate to update those fields in the existing set

        Returns: (application/json) either the new token and it's associated metadata
                                    or an error message (see below) 

  * DELETE /token
        Remove the token and it's metadata

        Returns: 202 Accepted


  * HEAD /dump
        Request the server to dump all metadata and the associated expiration timestamp
        to the server log

        Returns: 202 Accepted

### Metadata
Both the POST and PUT request accept a JSON body, which must contain a "meta" key,
which in turn must be a single JSON object.
```json
    {
        "meta": {
            "name": "My user",
            "year": 2022,
            "period": 11
        } 
    }
```

### PUT result

  * on success:
```json
    {
        "Ok": {
            "token": "XXXX",
            "meta": {
                "name": "My user",
                "year": 2022,
                "period": 11
            }
        } 
    }
```
  * on error:
  ```json
    {
        Err: "InvalidToken"
    }
```
