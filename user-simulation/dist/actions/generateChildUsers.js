"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenerateChildUsers = void 0;
const user_1 = require("../types/user");
const fs = __importStar(require("fs"));
const dotenv = __importStar(require("dotenv"));
dotenv.config();
// GenerateChildUsers action
class GenerateChildUsers {
    async execute(api, keyring, user, count) {
        const childUsers = [];
        const filePath = './generated_users.txt';
        // Use a base seed for generating child accounts
        const baseSeed = process.env.BASE_SEED || '//User';
        for (let i = 0; i < count; i++) {
            // Generate a unique seed for each child user
            const childSeed = `${baseSeed}/Child/${i}`;
            const childKeyPair = keyring.addFromUri(childSeed);
            const childUser = new user_1.User(childKeyPair);
            await childUser.updateBalance(api);
            childUsers.push(childUser);
            console.log(`Generated child user ${i}:
                Address: ${childUser.address}
                Seed: ${childSeed}
                Initial Balance: ${childUser.balance}`);
            // Append user data to the file
            const userData = `Child User ${i} - Address: ${childUser.address}, Seed: ${childSeed}, Balance: ${childUser.balance}\n`;
            fs.appendFileSync(filePath, userData);
        }
        console.log(`User data written to ${filePath}`);
        return childUsers;
    }
}
exports.GenerateChildUsers = GenerateChildUsers;
