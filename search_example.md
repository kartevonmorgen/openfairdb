# Search Example

```sh
curl -X GET 'https://nightly.ofdb.io/v0/search?text=%23teikei&categories=2cd00bebec0c48ba9db761da48678134,77b3c33a92554bcf8e8c2c86cedd6f6f&bbox=47.97337538577628,10.95611572265625,48.29004635269247,12.274475097656252'
curl -X GET 'http://localhost:8000/search?text=%23teikei&categories=2cd00bebec0c48ba9db761da48678134,77b3c33a92554bcf8e8c2c86cedd6f6f&bbox=47.97337538577628,10.95611572265625,48.29004635269247,12.274475097656252'
```

```json
{
  "visible": [
    {
      "id": "4a28e38695854059a457beb3b53c2578",
      "lat": 48.1313764,
      "lng": 11.6786861
    },
    {
      "id": "6d6146e07f584e908d142e15a3895917",
      "lat": 48.1113642,
      "lng": 11.5949569
    },
    {
      "id": "636eccbedf4243798351bdc3265c76db",
      "lat": 48.115337100000005,
      "lng": 11.5759799
    }
  ],
  "invisible": [
    {
      "id": "fe2e835bb28e4082b0dbd8f2a7158d4f",
      "lat": 51.95532468103803,
      "lng": 7.637477517127992
    },
    {
      "id": "de6ac9a92bde46d0b34bd5961196ec12",
      "lat": 47.4913731,
      "lng": 7.6175922
    },
    {
      "id": "67dd3653f45c4deaa961c0369520219c",
      "lat": 19.471123927418738,
      "lng": -96.96337670087816
    },
    {
      "id": "ff20f44776c0486682bc2e926d6d2bb3",
      "lat": 48.7244278,
      "lng": 9.14492801713147
    },
    {
      "id": "c5c844f92a4d4e459392197bc6805ee7",
      "lat": 47.9941572,
      "lng": 7.84340860047771
    }
  ]
}
```

<https://nightly.ofdb.io/v0/search?text=&categories=2cd00bebec0c48ba9db761da48678134,77b3c33a92554bcf8e8c2c86cedd6f6f&bbox=47.48008846346322,-3.0761718750000004,53.94315470224928,24.916992187500004>

curl -X GET '<http://localhost:8000/search?text=&categories=2cd00bebec0c48ba9db761da48678134>,77b3c33a92554bcf8e8c2c86cedd6f6f&bbox=47.48008846346322,-3.0761718750000004,53.94315470224928,24.916992187500004'

curl -X GET '<http://localhost:8000/search?text=&categories=2cd00bebec0c48ba9db761da48678134>,2cd00bebec0c48ba9db761da48678134,77b3c33a92554bcf8e8c2c86cedd6f6f&bbox=47.48008846346322,-3.0761718750000004,53.94315470224928,24.916992187500004'

4 visible + 5 invisible
<http://localhost:8000//search?text=&categories=2cd00bebec0c48ba9db761da48678134,77b3c33a92554bcf8e8c2c86cedd6f6f&bbox=52.4,8.5,52.5,12.5&limit=5>
