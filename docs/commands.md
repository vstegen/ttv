# Commands

## Auth

The `auth` command will fetch a new app access token irrespective of whether the current one is still valid or not.
For the command to succeed, a valid client ID and client secret are required to be present in the configuration.
These can be set by running `config`.

## Config

The `config` command allows to specify the client ID and client secret used to make API calls to Twitch.tv.
It can also store the app access token and its expiry.
Use `--show` to print the current configuration (secrets are masked).

## Follow

The `follow` command allows to (locally) follow (multiple streamers). Following only happens locally and your follows on Twitch.tv are unaffected by this.

## List

The `list` command lists all the streamers you follow. It allows filtering by the current status (`offline`, `online`,
`all`). By default, it will list only the streamers who are currently online.

It allows sorting by stream category via a `sort` boolean flag.

## Unfollow

The `unfollow` command allows you to remove a local follow of (multiple) streamers.

## VOD

The `vod` command allows specifying the streamer you want to watch a VOD of.
When not supplying an argument, all followed streamers will be listed
and `ttv` will allow selecting from the available streams.

## Watch

The `watch` command allows to specifiy (multiple) streamers for whom you want to start their stream.
If no streamer is specified, you will get a list of all online streamers you are following.

It allows sorting by stream category via a `sort` boolean flag.

Streams are started via `streamlink`
