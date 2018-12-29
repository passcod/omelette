# omelette

[![Build status](https://flat.badgen.net/travis/passcod/omelette/master)](https://travis-ci.com/passcod/omelette)

Inspired by [@MylesBorins’s cleanup tool](https://github.com/MylesBorins/cleanup).

The original idea is simple: given a tweet calling the bot with the hashtag
`#cleanup` and a time delay in seconds to hours, delete the entire thread, that
is, go up the reply chain and stop after the first tweet that either has no
parent or whose parent is someone else’s tweet (we can only delete our own).

Unfortunately, the original stopped working when Twitter disabled its stream API.

So this conceptually started as a re-implementation. But while I want to cleanup
_public_ history, I kinda want to keep it for myself. I might also want to do
more powerful or extensive stuff, like cleaning up all branches of a thread, or
deleting my entire history up to a point, and then keep deleting as I keep
tweeting, essentially keeping me with a set amount of history, like 6 months or
something, and archiving the rest. And as I worked, more possibilities emerged.

## so that’s the story. what does this do?

Omelette is a collection of small tools:

 - `omelette-sync` fetches from the Twitter API and stores a copy of all your
   own tweets, or as far as it sees them, plus media entity metadata.

 - `omelette-delete` processes deletions requests.

 - `omelette-migrate-db` prepares the database.

 - `omelette-mediatise` retrieves media content from entities and stores it all
   locally, so you can also archive/backup all photos, videos, GIFs, etc.

 - `omelette-cleanup` parses the database for `#cleanup` requests and figures
   out which tweets and threads to request deletion for.

 - [`omelette-twitter-archive`](#twitter-archive) imports tweets from a Twitter
   Archive file, either directly from the zip or from the extracted tweets.csv,
   then returns to the Twitter API to hydrate the tweets aka fill in the details
   (the archive data is very sparse).

 - [`omelette-twitter-events`](#twitter-events) is a web server that sets up and
   consumes account activity webhook events, stores incoming tweets, and can
   trigger other omelette tools in turn. **(TODO)**

You can bolt on additional behaviour simply by running a script or tool of your
own that reads statuses from and writes deletion requests to the database.
Please contribute useful tools back to this repo!

## how do i run it?

Like all twitter bots, this needs keys. It’s to be used only in a personal
capacity, so there’s no need to do lots of OAuth. Just sign up for an app, get
your tokens, put them into your environment:

```
TWITTER_CONSUMER_KEY=
TWITTER_CONSUMER_SECRET=
TWITTER_ACCESS_TOKEN_KEY=
TWITTER_ACCESS_TOKEN_SECRET=
```

Then you’ll need a Postgres database. There’s not gonna be much load on it, so
just spin one right next to your app, or whatever is most convenient, create a
database the usual way, put the URL in your environment:

```
DATABASE_URL=postgres://localhost/dbname
```

You’ll also need your user ID. If you don’t know it, you can look it up using
any of a number of services, like this one: https://tweeterid.com/

```
TWITTER_USER_ID=
```

At the first run, and after upgrades, you’ll need to set up the database:

```bash
omelette-migrate-db
```

And finally, run omelette:

```bash
omelette-sync &&\
omelette-mediatise &&\
omelette-cleanup &&\
omelette-delete --dry-run
```

## …which i install how?

If you’ve got macOS, Windows, or Linux, head on over to [the releases tab].

Otherwise, if you’ve got Rust installed, clone and `cargo build --release`!

[the releases tab]: https://github.com/passcod/omelette/releases

## any general tips?

Pass the `--dotenv` flag to load from a `.env` file in the current directory.

Run `omelette-delete` with `--dry-run` for a few days or weeks before trusting
it to do the right thing. You can run it in `--interactive` mode once in a while
during that time to get prompted before deleting each tweet.

Every time there’s an update to omelette, do that again. There’s no undo, no way
to insert tweets back where they were again, so be careful with it.

Most tools with omelette are designed to be run at intervals, they’re not
daemons. Use crons or systemd timers to run them every so often. You’ll want to
pick a rate that doesn’t hit the API too much, while still being useful. This
varies by tool, but 5 minutes is often a good default. You may also hook them
to an omelette event daemon (see below).

## got more docs?

### twitter-archive

This tool works on a downloaded [Twitter archive file], which can be requested
from Twitter and will be emailed to you (warning: in some known cases,
high-volume tweeters have been unable to get their archive file).

If you’ve already extracted your archive, you can point the tool the tweets.csv
file. Otherwise, pointing the tool to the zip file will work too, at a slight
speed penalty. The tool batches and streams the import, so it will not eat all
your memory even if you have lots of history.

The import has two steps: the “slim” pass, and the “hydrate” pass.

In the “slim” pass, the tweets are read from the archive into the database. But
the archive is very light on details and contains ambiguous information in many
cases, so the tool only stores what it is sure of.

In the “hydrate” pass, the tool goes back to the Twitter API and looks up tweets
from the service, requesting the full set of information and filling the gaps in
the database. If it cannot find a tweet, it marks it as deleted in the database.

Because archives can contain lots of tweets, this pass is pre-emptively
throttled to batches of 100 tweets every 5 seconds, which means it can take a
long time to hydrate all tweets! For this reason, the tool can be stopped at any
time and restarted using the `--only-hydrate` flag, which will skip the first
pass and keep hydrating remaining tweets.

The `--only-slim` flag can be used to only run the first pass, if you know you
don’t have time or are on a metered internet connection, for example.

[Twitter archive file]: https://help.twitter.com/en/managing-your-account/how-to-download-your-twitter-archive

### twitter-events

This tool is a daemon that serves a web service and registers it as a webhook on
your user account. Twitter then delivers account activity events as HTTP POST
requests to the server. When the tool shuts down it will unregister the webhook.

If you have Tor installed and configured, and pass the `--tor` flag, the tool
will attempt to set up an onion service for its server, and register the webhook
for `onionservicename.onion.to`, making it possible to run behind a firewall.

Otherwise, you’ll need to run the tool on a server with a public interface, or
rig up a tunnel somehow (for example using [ngrok](https://ngrok.com), but note
ngrok’s limits may not suffice, as each event will count for one connection).

You can pass the `--public <URL>` option to tell the tool about the public URL
for the service, otherwise it will attempt to determine its own public IP, and
will error if it cannot successfully establish a public service.

The `--on-<event> <TOOL>` series of options will run the named tool (which can
be an omelette tool or any other program in PATH) when an event is received, but
after the event has been processed internally first (i.e. tweets added to the
database, etc). The environment variable `OMELETTE_STREAM_EVENT` will be set,
and the payload will be piped to the tool’s STDIN in JSON format.

For example, you’ll probably want to run `--on-create omelette-cleanup`.

There’s also a special event `--on-boot` that runs after the webhook service has
been configured, registered, and connectivity has been checked by Twitter.
You can find the whole list of events with `--help`.

## who do i have to thank for this?

There’s me, [@passcod](https://passcod.name).

There’s [@MylesBorins](https://mylesborins.com) who started the whole thing.

There’s [Diesel](http://diesel.rs) and [Egg Mode](https://github.com/QuietMisdreavus/twitter-rs) that do the heavy lifting.

## any last words?

This work is made available to you under the terms of the [Artistic 2.0] license.
Don’t be a jerk. Additionally, the [Contributor Covenant] applies.

[Artistic 2.0]: ./LICENSE
[Contributor Covenant]: https://www.contributor-covenant.org/version/1/4/code-of-conduct

This may contain bugs. One of the reasons it saves to DB is to avoid complete
disaster if it deletes stuff it’s not supposed to. But just in case, I advise
running with the `--dry-run` flag for a few weeks, before trusting it, and after
upgrades. I am not responsible for you losing your tweets. See clause 14.
