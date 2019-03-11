## v0.5.1-dev (2019-mm-dd)

- new(web): Add a basic admin frontend
- new(web): Include address fields for searching entries
- new(web): Archive events/entries/ratings/comments (admin only)
- fix(db):  Send registration e-mail
- fix(db):  Apply bounding box filter when searching
- chore(db): Type safe handling of passwords and timestamps

## v0.4.5 (2019-03-07)

- Final and official v0.4.x release due to technical reasons
- No functional changes since v0.4.4

## v0.5.0 (2019-03-04)

- new(web): Return additional properties of entries in search results
- new(web): Limit max. number of search results
- new(web): Bundle a basic frontend with a minimum of JavaScript
- chore(db): Add Tantivy full-text search engine to improve performance

## v0.4.4 (2019-02-17)

- fix: Never resolve the location of an event with an empty event address

## v0.4.3 (2019-02-15)

- fix: Resolve event location from address via geocoding/Opencage

## v0.4.2 (2019-02-14)

- fix: implicitly check and set lat/lng when creating or updating events
- fix: patch geocoding crate to fix OpenSSL system dependency issues
- fix: Makefile for Docker build
- fix: OpenSSL v1.1.1 build issues
- fix: truncate username if created from email
- fix: formatting for geocoding requests
- chore: update diesel: 1.3.3 -> 1.4.0
- chore: update dependencies & rustc
- doc: describe how to render the API docs

## v0.4.1 (2019-01-18)

- new: add `organizer` field to events
- new: forbid creating entries with owned tags
- new: check for lat/lng on a PUT request
- new: check event tags before creation
- fix: update event-tag relations
- chore: update dependencies

## v0.4.0 (2019-01-17)

- new: Event API
- new: OpenAPI documentation
- new: add PlantUM: class diagram
- new home: [slowtec/openfairdb](https://github.com/slowtec/openfairdb)
- chore: update `rocket` to `v0.4.0`
- fix: login

## v0.3.9 (2018-10-24)

- new(web): auto-complete URL fields
- fix(email): consistent corporate naming "Karte von morgen"
- fix(email): write RFC 2047 Subject header
- chore(db): add indexes for foreign key relations
- chore(*): update various dependencies

## v0.3.8 (2018-08-27)

- new(db): add image URL fields to entries
- chore(web): update `rocket` to `v0.3.16`
- chore(db): update `diesel` to `v1.3.x`
- chore(*): update dependencies

## v0.3.7 (2018-05-22)

- chore(web): update `rocket` to `v0.3.11`
- chore(*): update dependencies

## v0.3.6 (2018-05-13)

- new(csv-export): export average rating of an entry
- chore(web): update `rocket` to `v0.3.10`
- chore(*): update dependencies

## v0.3.5 (2018-04-27)

- new(csv-export): export category names instead of IDs
- chore(web): update `rocket` to `v0.3.9`

## v0.3.4 (2018-04-24)

- new(csv-export): implement csv-export of entries for a given bbox
- new(tags): ignore `#` char in tags
- new(cli): log info message when calculating average ratings is finished
- refactor(*): reorganize files
- chore(*): update `rocket` to `v0.3.8`
- chore(*): update dependencies to compile on latest nightly

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
