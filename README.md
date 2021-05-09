# bq2cst
Parse standard SQL, which is a dialect of [BigQuery](https://cloud.google.com/bigquery), into a concrete syntax tree.

⚠️ **This parser is designed to be used via [prettier-plugin-bq](https://github.com/dr666m1/project_prettier_bq).**

## Features
- forcused on standard SQL (in other words, other SQL dialects are out of scope)
- developed in Rust, using [wasm-pack](https://github.com/rustwasm/wasm-pack)

## Install
```shell
npm install @dr666m1/bq2cst
```

## Usage
```javascript
const parser = require("@dr666m1/bq2cst");
parser.parse("SELECT 1;")

//[
//  {
//    "token": {
//      "line":1,
//      "column":1,
//      "literal":"SELECT"
//    },
//    "node_type":"SelectStatement",
//    "children":{
//      "semicolon":{
//        "Node":{
//          "token":{"line":1,"column":9,"literal":";"},
//          "node_type":"Symbol",
//          "children":{}
//        }
//      },
//      "exprs":{
//        "NodeVec":[{
//          "token":{"line":1,"column":8,"literal":"1"},
//          "node_type":"NumericLiteral",
//          "children":{}
//        }]
//      }
//    }
//  },
//  {
//    "token":null,
//    "node_type":"EOF",
//    "children":{}
//  }
//]
```
