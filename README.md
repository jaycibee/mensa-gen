# mensa-gen
> a canteen plan scraper and generator for [Studierendenwerk Hamburg](https://stwhh.de) locations

you can see it in action [here](https://gauss.love/mensa)

## usage
```
$ ./mensa-gen [-l <LOCATIONS>] <TEMPLATE> <OUTFILE>
```

This scrapes the mensa plans for today, optionally restricting locations to those included in the `LOCATIONS` file and rendering the [tera](https://github.com/Keats/tera) template `TEMPLATE` to the file `OUTFILE`

## templates
mensa-gen generates pages based on a [tera](https://github.com/Keats/tera) templates. the data is available through a `locations` array containing items with the following structure:
```
{
    "name": "the name of the location",
    "meals": [
        {
            "name": "the name of the meal",
            "price": "the price of a meal (as a string with , as a delimiter)",
            "kind": "one of 'Normal', 'Vegetarian' or 'Vegan'"
        },
        ...
    ]
}
```
