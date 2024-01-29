const express = require("express");
const app = express();
const port = 3000;
var bodyParser = require("body-parser");
const { exec } = require("node:child_process");
const fs = require('fs');
const path = require("node:path");

app.use(bodyParser.json());

const circomPath = path.join(__dirname, '../target/debug/circom');

app.get("/", (req, res) => {
  let { file } = req.body;
  //TODO: make random number more "secure"
  const r = Math.floor(Math.random() * Date.now());

  const filePath = path.join(__dirname, 'assets', `${r.toString()}.circom`); 
  fs.writeFile(filePath, file, (err) => {
    exec(`${circomPath} ${filePath}`, (error, stdout, stderr) => {
        if (error) {
          console.error(`exec error: ${error}`);
          return;
        }
        console.log(`stdout: ${stdout}`);
        console.error(`stderr: ${stderr}`);
      });
    res.send({"ok": 1})
  });
 

});

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`);
});
