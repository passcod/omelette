# omelette

Inspired by [@MylesBorins’s cleanup tool](https://github.com/MylesBorins/cleanup).

The original idea is simple: given a tweet calling the bot with the hashtag
`#cleanup` and a time delay in seconds to hours, delete the entire thread, that
is, go up the reply chain and stop after the first tweet that either has no
parent or whose parent is someone else’s tweet (we can only delete our own).

Unfortunately, the original stopped working when Twitter disabled its stream API.

So this conceptually started as a re-implementation. But while I want to cleanup
_public_ history, I kinda want to keep it for myself. And I might want to do
more powerful or extensive stuff, like cleaning up all branches of a thread, or
deleting my entire history up to a point, and then keep deleting as I keep
tweeting, essentially keeping me with a set amount of history, like 6 months or
something, and archiving the rest.

## so that’s the story. what does this do?

Omelette is a collection of small tools:

 - `omelette-sync` fetches and stores a copy of all your own tweets, or as far
   as it sees them, plus media entity metadata, in a PostgreSQL database.

 - `omelette-delete` processes deletions requests.

 - `omelette-mediatise` retrieves media content from entities and stores it all
   locally, so you can also archive/backup all photos, videos, GIFs, etc.

 - `omelette-cleanup` parses the database for `#cleanup` requests and figures
   out which tweets and threads to request deletion for.

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

And finally, run omelette:

```bash
omelette-sync &&\
omelette-mediatise &&\
omelette-cleanup &&\
omelette-delete --dry-run
```

## …which i install how?

Well, if you’ve got Rust installed, a simple `cargo install omelette` will do.

If you don’t, or you prefer a binary, head on over to [the releases tab].

[the releases tab]: https://github.com/passcod/omelette/releases

## any more tips?

Yeah.

Pass the `--dotenv` flag to load from a `.env` file in the current directory.

Run `omelette-delete` with `--dry-run` for a few days or weeks before trusting
it to do the right thing. You can run it in `--interactive` mode once in a while
during that time to get prompted before deleting each tweet.

Every time there’s an update to omelette, do that again. There's no undo, no way
to insert tweets back where they were again, so be careful with it.

Omelette is designed to be run at intervals, it's not a daemon. Use a cron or a
systemd timer to keep it going.

If you don’t tweet much, you don’t need to run it as much. But if you tweet lots,
you'll need to run it more often. When it syncs, it will tell you how many calls
it had to make to the Twitter API to go however far back as it needed to pull
down your tweet history since it last ran. You want to get it so that it does
one call every time it runs, and no more than that. Give yourself some margin to
account for peaks and bursts, but otherwise, set intervals as wide as you can
while still being often enough to be useful (15 minutes at most, plausibly.)

## that’s cool. who do i have to thank for this?

There’s me, [@passcod](https://passcod.name).

There’s [@MylesBorins](https://mylesborins.com) who started the whole thing.

There’s [Diesel](http://diesel.rs) and [Egg Mode](https://github.com/QuietMisdreavus/twitter-rs) that do the heavy lifting.

## any last words?

Sure.

This work is made available to you under the terms of the [Artistic 2.0] license.
Don’t be a jerk. Additionally, the [Contributor Covenant] applies.

[Artistic 2.0]: ./LICENSE
[Contributor Covenant]: https://www.contributor-covenant.org/version/1/4/code-of-conduct

This may contain bugs. One of the reasons it saves to DB is to avoid complete
disaster if it deletes stuff it’s not supposed to. But just in case, I advise
running with the `--dry-run` flag for a few weeks, before trusting it, and after
upgrades. I am not responsible for you losing your tweets. See clause 14.
