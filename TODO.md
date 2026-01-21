# TODO

## To finish up v1.0.0:

- Fill in more of the documentation I want in both README and capsule.
- Fix bug where client hangs when proxy fails, not sure of this is client side proxy side, but doesn't happen direct without proxy.
- Fix bug where currently --cache and --store command line seem to be ignored or overwritten with default.
- Add the packaging and build code for the release.

## Future After v1.0.0 Release:

- Add help mode with --mode help and --help, as well as h key for displaying help within client.
- Currently, when cert doesn't match cache, client errors and exits, and other failures end, instead display error within client.
- Add setup mode that prompts for which mode to configure, then builds the config file, instead of create on missing.
- Comments on the code.
- While file system and client (for proxy or for single gemini server) backeps work, adding the code for http/https backend, which will facilitate using things like S3, github, bitbucket, IPFS, etc as the backend.
- Serve gemtext as HTML in a built in web server,
- HTML from http/https origin served as gemtext. Not sure on either of these if I want to go there.
- Implementing dynamic requests in the server, either by spawning backend processes to generate content or some type of in app templating.
- Implementing the client side handling of 1X responses and prompting for dynamic content.
