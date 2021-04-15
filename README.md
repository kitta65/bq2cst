# bq2ast
Parse standard SQL, which is a dialect of [BigQuery](https://cloud.google.com/bigquery), into a concrete syntax tree.

⚠️ **This parser is designed to be used via [prettier-plugin-bq](https://github.com/dr666m1/project_prettier_bq).**

## Features
- forcused on standard SQL (in other words, other SQL dialects are out of scope)
- developed in Rust, using [wasm-pack](https://github.com/rustwasm/wasm-pack)

## Install
```
npm install @dr666m1/bq2cst
```

## Usage
```
const parser = require("@dr666m1/bq2cst");
parser.parse("select *;")
```
