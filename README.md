# Lupercal

A simple Gemini server and client written in Rust, also including a proxy mode and ability to convert capsules to HTML.

## Behind the Name

Gemini is named for the NASA space program and crafts that were the first two man spacecrafts, that name was a play off the constellation. Gemini means "twin", or "twinborn", for Gemini is the twins from myth, Caster and Pollux. The space program was named after thr constellation because of having two crew members, verses the previous Mercury with a single astronaunt and the upcoming Apollo missions that would have three. The protocol was named for Gemini because Gemini was the bridge from the initial flights of Mercury to the intended flights of Apollo. The Gemini protocol forms a bridge between the very simple Gopher Protocol that forms Gopherspace and the far more complex HTTP and HTTPS protocol that form the World Wide Web.

Lupercal is a play off the use of Gemini as the twins. It represents a different pair of twins, Romulus and Remus, the founders of Rome. Lupercal is a form of Lupercus, from lupus, "wolf" and arceō, "to fend off". Lupercal or Lupercalia was a Roman festival and sacrifice to Lycaean Pan, also called Faunus in Latin, every February. The sacrifice occurred in the cave in which it was believed the shewolf nursed Romulus and Remus, so the festival and sacrifice held a dual meaning, Pan or Faunus as both the wolf that consumes and the protector against the wolf as well as the protection and nursing of the founding twins of Rome.

## Usage

All modes can be ran from the lupercal binary, which defaults to client if no mode it specified.

```
lupercal --mode [server|client|proxy|convert] [options] [url]
```

Use the options below depending on which mode Lupercal is ran as:

### Client

In client mode, Lupercal runs as a client to request content from gemini servers.

```
lupercal [options] url
lupercal --mode client [options] url
lupercal-client [options] url

  url is the path requested, in the form gemini://hostname/path

  Options:
    --config [config file]              Config file setting any of the other options.
    --cache  [cache directory]          Specify a custom cache directory.
    --store  [key store directory]      Specify a custom key store directory, defaults as store under the cache.
    --name   [username@host or email]   The client name, used in the client certificate.
    --proxy  [proxy server name]        The hostname of a proxy server if required.
    --allow  [list of hostnames]        If present, limits client requests to only hostnames in the list, useful for kiosk style usage. Comma separated.
    --deny   [list of hostnames]        If present, blocks client requests that are to hostnames in the list, useful to block undesirable sites. Comma separated.
    --lock                              Lock the private key, requiring passprase on future runs.
    --unlock                            Unlock the private key, to not require passprase on future runs (warning, insecure mode).
```

### Server

In server mode, Lupercal runs as a gemini server.

```
lupercal --mode server [options]
lupercal-server [options]

  Options:
    --config [config file]              Config file setting any of the other options.
    --cache  [cache directory]          Specify a custom cache directory.
    --store  [key store directory]      Specify a custom key store directory, defaults as store under the cache.
    --name   [server hostname]          The server hostname, used in the server certificate, and the hostname requests are requested for.
    --source [source]                   The directory to pull content from, or alternatively and http, https, or gemini URL to pull from.
    --listen [IP address]               Specify a specific IP address to listen on.
    --lock                              Lock the private key, requiring passprase on future runs.
    --unlock                            Unlock the private key, to not require passprase on future runs (warning, insecure mode).
```

### Proxy

In proxy mode, Lupercal runs a server that accepts requests for other Gemini servers, then forwards the request to the other server, returning the response back to the client. Use the --proxy option in the client to use a proxy.

```
lupercal --mode proxy [options]
lupercal-proxy [options]

  Options:
    --config [config file]              Config file setting any of the other options.
    --cache  [cache directory]          Specify a custom cache directory.
    --store  [key store directory]      Specify a custom key store directory, defaults as store under the cache.
    --name   [proxy hostname]           The proxy hostname, used in the proxy certificate.
    --allow  [list of hostnames]        If present, limits proxy requests to only hostnames in the list. Comma separated.
    --deny   [list of hostnames]        If present, blocks proxy requests that are to hostnames in the list. Comma separated.
    --lock                              Lock the private key, requiring passprase on future runs.
    --unlock                            Unlock the private key, to not require passprase on future runs (warning, insecure mode).
```

### Convert

Convert a gemini capsule to a simple HTML website.

```
lupercal --mode convert --in [gemini directory] --out [html directory]
lupercal-convert --in [gemini directory] --out [html directory]

  Options:
    --in [gemini directory]             Gemini capsule directory to convert.
    --out [HTML directory]              HTML directory to place the converted site into.
```