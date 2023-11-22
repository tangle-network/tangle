const fs = require('fs');
const path = require('path');

// Read the JSON file
const data = fs.readFileSync(path.join(__dirname, 'edg-balances-new.json'));
const balances = JSON.parse(data);

// Calculate the total balance
let totalBalance = 0;
for (let account in balances) {
  totalBalance += balances[account].Total;
}

// Calculate the fraction of each account's balance over the total
let fractions = {};
for (let account in balances) {
  if (account in fractions) {
    fractions[account] = fractions[account] + (balances[account].Total / totalBalance);
  } else {
    fractions[account] = balances[account].Total / totalBalance;
  }
}

// Write the output to a new JSON file
fs.writeFileSync('output.json', JSON.stringify(fractions, null, 2));
