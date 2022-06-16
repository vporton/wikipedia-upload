const { Bee } = require("@ethersphere/bee-js");
const ethers = require('ethers');  
const crypto = require('crypto');
const fs = require('fs');
var axios = require('axios').default;

if(process.argv.length != 4) {
    console.log("Usage: node signfeed-cli.js <feedTopic> <feedValue>\n\n" +
      "<feedValue> is a JSON object")
}

const feedTopic = process.argv[2];
const feedValue = process.argv[3];

let privateKey;
const keyFilename = '/private/feedSignKey.private';
try {
    privateKey = fs.readFileSync(keyFilename);
} catch(e) {
    privateKey = "0x" + crypto.randomBytes(32).toString('hex');
    // fs.writeFileSync(keyFilename, privateKey);
}

(async function(){
  const bee = new Bee('http://localhost:1633')

  const res = await axios.post("http://localhost:1635/stamps/100/17")
  const postageBatchId = res.data['batchID'];

  await bee.setJsonFeed(
    postageBatchId,
    feedTopic, 
    JSON.parse(feedValue), 
    { signer: privateKey }
  )
})();