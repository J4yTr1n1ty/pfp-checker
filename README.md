# Profile Picture Checker

A Discord bot to provide a Profile Picture checker to see how many times someone changes their Profile Picture.

## Features

- Add Users to monitor
- Check at a specified interval if the profile picture has changed and saves the new profile picture.
- Shows the history of profile pictures
- Shows the amount of changes since starting to monitor, changes per month/week/year


## ToDos
- [ ] Add Database integration with local SQLite Database (https://github.com/serenity-rs/serenity/tree/current/examples/e16_sqlite_database)
- [ ] Add Cronjob to monitor Users (https://docs.rs/cronjob/latest/cronjob/)
- [ ] Add Remove Monitor Command
- [ ] Add History Command
    - Previous Profile Pictures
    - Duration of usage
- [ ] Add Stats Command
    - Amount of changes since monitor
    - Start of Monitor
    - Average changes per week, month and year
