## v0.3.3 (2018-02-23)

- fix(web): login with username instead of user id
- fix(web): create new account

## v0.3.2 (2018-02-12)

- fix(web): ignore tag duplicates
- fix(web): subscribe to bbox
- fix(web): email-confirmation
- fix(web): transform tags to lowercase
- refactor(test): use sqlite instead of mock db

## v0.3.1 (2018-01-21)

- fix(db): add tag relations
- fix(test): make coveralls on travis work

## v0.3.0 (2018-01-19)

- new(db): remove neo4j support (SQLite is now required)
- new(db): add functionality to import OSM nodes
- new(*): Improve sorting & search algorithms
- refactor(db): freeze initial DB-Schema
- refactor(*): remove tripples (make relations explicit)
- refactor(*): tidy up & format code & rename some functions
- chore(email): make email functionality optional
- chore(web): update `rocket` to `v0.3.6`
- chore(web): update `diesel` to `v1.1.1`
- fix(*): use logging level from environment
- fix(sort): don't overflow on calculating the average rating

## v0.2.12 (2018-01-09)

- fix(*): use logging level from the environment

## v0.2.11 (2018-01-08)

- new(search): improve performance
- revert(db): use neo4j by default
- chore(*): update `rocket` to `v0.3.5`

## v0.2.10 (2017-12-06)

- new(db): use [SQLite](https://sqlite.org/) and [diesel](https://diesel.rs)
  instead of [neo4j](https://neo4j.com/)

## v0.2.9 (2017-09-16)
- new(api): always respond with 'application/json'
- fix(emails): fix encoding of emails
- new(emails): change links in emails from prototype to main app
- new(register): change character limit of username to 30
- new(search): use AND for tags in search

## v0.2.8 (2017-09-21)

- new(accounts): add the possibility to register and login
- new(subscribe): add the possibility to subscribe to a bbox when logged in

## v0.2.7 (2017-07-03)

- include search results where tags match search words (also when they are not preceded by "#")
- give the possibility to add sources when rating an entry
- add basic mail notification (to emails in a config file)

## v0.2.6 (2017-05-26)

- new(ranking): calculate average rating of an entry in each rating context separately
- new(ranking): calculate average rating of an entry by taking average of all rating contexts
- new(ranking): rank search results by average rating

## v0.2.5 (2017-05-19)

- fix: revert `references` path
- fix: defer proper logging with rocket

## v0.2.4 (2017-05-19)

- new: add `references` to rating struct
- new: `/login`
- new: `/logout`
- chore: use git master branch of `rocket`

## v0.2.3 (2017-04-12)

- new basic rating support
- new: `GET /tags`
- new: `user` entity + account creation
- new: disallow fetching all entries at once

## v0.2.2 (2017-03-23)

- new: search by hash tags
- new: `/count/tags`
- new: `categories` is now optional in `/search?`
- fix: `/count/entries`
- refactor: use `State` to manage DB connections
- chore: update `rocket` to `v0.2.3`

## v0.2.1 (2017-03-22)

- new: filter by tags

## v0.2.0 (2017-03-19)

- new: basic tagging
- refactor: use a verbose DB trait

## v0.1.1 (2017-03-10)

- several fixes to make the API compatible to `v0.0.16` again
- chore: update `rocket` to `v0.2.x`
- chore: use `r2d2-cypher` form crates.io

## v0.1.0 (2017-02-04)

- refactor: clean architecture
- refactor: use [rocket](https://www.rocket.rs)
- remove: CORS support

## v0.0.x

Please run `git log --pretty=oneline --abrev-commit v0.0.16` ;-)
