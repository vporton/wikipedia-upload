const Bee = require("@ethersphere/bee-js");
const ethers = require('ethers');  
const crypto = require('crypto');
const fs = require('fs');

if(process.argv.length != 2) {
    console.log("Usage: signfeed-cli.js <feedTopic> <feedValue>\n\n" +
      "<feedValue> is a JSON object")
}

let privateKey;
const keyFilename = '/private/feedSignKey.private';
try {
    privateKey = fs.readFileSync(keyFilename);
} catch(e) {
    privateKey = "0x" + crypto.randomBytes(32).toString('hex');
    fs.writeFileSync(keyFilename, privateKey);
}

(async function(){
  const bee = new Bee('http://localhost:1633')

  const postageBatchId = await bee.createPostageBatch("100", 17)

  await bee.setJsonFeed(
    postageBatchId,
    feedTopic, 
    JSON.parse(feedValue), 
    { signer: privateKey }
  )
})();