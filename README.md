# Profile Picture Spy

Do you also have some friends on Discord that change their profile pcitures way too often? Ever wondered how many times they change their profile pictures? 
This bot will be your _blazingly fast_ solution.

This is a Discord bot to provide a Profile Picture History to see how many times someone changes their Profile Picture.

## Features

- Add Users to monitor
- Check at a specified interval if the profile picture has changed and saves the new profile picture.
- Shows the history of profile pictures
- Shows the amount of changes since starting to monitor, changes per month/week/year

## ToDos
### v0.2.0
- [x] Add Database integration with local SQLite Database (https://github.com/serenity-rs/serenity/tree/current/examples/e16_sqlite_database)
- [x] Add Cronjob to monitor Users (https://docs.rs/cronjob/latest/cronjob/)
- [x] Add Remove Monitor Command
- [x] Add History Command
    - [x] Previous Profile Pictures

### v0.3.0
- [ ] Add Stats Command
    - Amount of changes since monitor
    - Start of Monitor
    - Average changes per week, month and year
