# Wellbeing Rankings

Exports the ranking data from [Wellbeing Rankins](https://www.nber.org/papers/w30759) by David G. Blanchflower & Alex Bryson into machine readable formats (JSON and TSV).

For context, here is the abstract of their paper.

> Combining data on around four million respondents from the Gallup World Poll and the US Daily Tracker Poll we rank 164 countries, the 50 states of the United States and the District of Colombia on eight wellbeing measures. These are four positive affect measures - life satisfaction, enjoyment, smiling and being well-rested – and four negative affect variables – pain, sadness, anger and worry. Pooling the data for 2008-2017 we find country and state rankings differ markedly depending on whether they are ranked using positive or negative affect measures. The United States ranks lower on negative than positive affect, that is, its country wellbeing ranking looks worse using negative affect than it does when using positive affect. Combining rankings on all eight measures into a summary ranking index for 215 geographical locations we find that nine of the top ten and 16 of the top 20 ranked are US states. Only one US state ranks outside the top 100 – West Virginia (101). Iraq ranks lowest - just below South Sudan. Country-level rankings on the summary wellbeing index differ sharply from those reported in the World Happiness Index and are more comparable to those obtained with the Human Development Index.

## Pre-requisites

- Rust (https://rustup.rs/)
- Poppler (https://poppler.freedesktop.org/)

## Extracting the data

The data is contained within tables inside the pdf of the paper (a [snapshot](w30759.pdf) is committed in this repo for convenience) and will be extracted by running,

```
cargo run
```

## Author

[Kelly Norton](https://github.com/kellegous/)