# Open Fair DB

This repository will contain the source code of the
**Open Fair DB** project some day.

## REST API

The current REST API is quite basic and will change within the near future.
The base URL is `http://api.ofdb.io/v0/`.

-  `GET /entries/:ID`
-  `GET /entries/:ID_1,:ID_2,...,:ID_n`
-  `POST /entries/`
-  `PUT /entries/:ID`
-  `GET /categories/`
-  `GET /categories/:ID`
-  `GET /search?text=TXT&bbox=LAT_min,LNG_min,LAT_max,LNG_max&categories=C_1,C_2,...,C_n`

#### JSON structures

The structure of an `entry` looks like follows:

```
{
  "id"          : String,
  "version"     : Number,
  "created"     : Number,
  "name"        : String,
  "description" : String,
  "lat"         : Number,
  "lng"         : Number,
  "street"      : String,
  "zip"         : String,
  "city"        : String,
  "email"       : String,
  "telephone"   : String,
  "homepage"    : String,
  "categories"  : [String]
}
```

The structure of a `category` looks like follows:

```
{
  "id"      : String,
  "version" : Number,
  "created" : Number,
  "name"    : String,
  "parent"  : String
}
```

# License

Copyright (c) 2015 Markus Kohlhase

This project is licensed unter the AGPLv3 license.
