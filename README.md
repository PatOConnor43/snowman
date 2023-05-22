# Snowman
A CLI tool for interacting with Postman.

**This software is very alpha and the API can change with any release.**

## Motivation
I really wanted a central place for my team to put API related values. This includes base URLs and common IDs. Postman's environments are great for this. We can have common environments and fork them to add user specific information if necessary. The _problem_ for me is that I _really_ like to use `curl` to make my requests. Making my requests in the shell allows me to do all kinds of scripting and parsing using jq, that I just don't like to do in Postman pre-request scripts or Tests. 

Using this tool I can activate an environment and make a request like this:
```
curl $SNOWMAN_DOMAIN/documents -X GET | jq '.'
```
If I need to change to an environment where the DOMAIN variable is different, I can activate that environment and make the exact same request.

## Features
- Inject Postman Environment values into shell environment variables
- Fuzzy find through workspaces and environments

## Setup
- Use the install script within the release page
- Copy your Postman cookie header from the browser
  - This can be done by opening the dev tools, going to the network tab, and copying the `cookie` request header from any request.
- Use `snowman config` to populate your config. This will open your $EDITOR with prepopulated keys. You'll need to fill in the `cookie` and `domain` values.
- Use `snowman activate` to find the environment you want to inject
- Use these environment variables to make `curl` requests or anything else you're interested in.

## Changelog
Look at the RELEASES file for information and release notes.

