# Open Fair DB

This repository will contain the source code of the
**Open Fair DB** project some day.

## REST API

The current REST API is quite basic and will change within the near future.
The base URL is `/api/v0/`.

-  `/entries/:ID`
-  `/entries/:ID_1,:ID_2,...,:ID_n`
-  `/categories/`
-  `/categories/:ID`
-  `/search?text=TXT&bbox=LAT_min,LNG_min,LAT_max,LNG_max&categories=C_1,C_2,...,C_n`

### JSON structure

The structure of an entry looks like follows:

```
{
  "id"          : Number,
  "title"       : String,
  "description" : String,
  "lat"         : Number,
  "lon"         : Number,
  "street"      : String,
  "zip"         : String,
  "city"        : String,
  "email"       : String,
  "telephone"   : String,
  "homepage"    : String,
  "categories"  : [Number]
}
```
