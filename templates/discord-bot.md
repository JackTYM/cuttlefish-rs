---
name: discord-bot
description: Discord.js bot with TypeScript and slash commands
language: typescript
docker_image: node:22-slim
variables:
  - name: project_name
    description: Name of the bot
    required: true
  - name: bot_name
    description: Display name for the bot
    default: "{{ project_name }}"
tags: [discord, bot, typescript]
---

# {{ project_name }}

Discord bot built with Discord.js and TypeScript featuring slash commands and event handlers.

## Project Structure

```
{{ project_name }}/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts
│   ├── config.ts
│   ├── commands/
│   │   ├── ping.ts
│   │   ├── help.ts
│   │   └── user-info.ts
│   ├── events/
│   │   ├── ready.ts
│   │   ├── interactionCreate.ts
│   │   └── messageCreate.ts
│   ├── utils/
│   │   ├── logger.ts
│   │   └── helpers.ts
│   └── types/
│       └── index.ts
├── dist/
├── Dockerfile
├── docker-compose.yml
└── .env.example
```

## Files

### package.json
```json
{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "description": "Discord bot - {{ bot_name }}",
  "main": "dist/index.js",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js",
    "lint": "eslint src --ext .ts",
    "format": "prettier --write src"
  },
  "dependencies": {
    "discord.js": "^14.14.0",
    "dotenv": "^16.3.1"
  },
  "devDependencies": {
    "@types/node": "^20.10.0",
    "@typescript-eslint/eslint-plugin": "^6.13.0",
    "@typescript-eslint/parser": "^6.13.0",
    "eslint": "^8.54.0",
    "prettier": "^3.1.0",
    "tsx": "^4.7.0",
    "typescript": "^5.3.3"
  }
}
```

### tsconfig.json
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ES2020",
    "lib": ["ES2020"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "moduleResolution": "node"
  },
  "include": ["src"],
  "exclude": ["node_modules", "dist"]
}
```

### src/index.ts
```typescript
import { Client, GatewayIntentBits, Collection } from 'discord.js';
import { config } from 'dotenv';
import { readdirSync } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { logger } from './utils/logger.js';

config();

const __dirname = fileURLToPath(new URL('.', import.meta.url));

interface ExtendedClient extends Client {
  commands: Collection<string, any>;
}

const client = new Client({
  intents: [
    GatewayIntentBits.Guilds,
    GatewayIntentBits.GuildMessages,
    GatewayIntentBits.MessageContent,
    GatewayIntentBits.DirectMessages,
  ],
}) as ExtendedClient;

client.commands = new Collection();

// Load commands
const commandsPath = join(__dirname, 'commands');
const commandFiles = readdirSync(commandsPath).filter(file => file.endsWith('.ts') || file.endsWith('.js'));

for (const file of commandFiles) {
  const filePath = join(commandsPath, file);
  const command = await import(filePath);
  if (command.default.data && command.default.execute) {
    client.commands.set(command.default.data.name, command.default);
    logger.info(`Loaded command: ${command.default.data.name}`);
  }
}

// Load events
const eventsPath = join(__dirname, 'events');
const eventFiles = readdirSync(eventsPath).filter(file => file.endsWith('.ts') || file.endsWith('.js'));

for (const file of eventFiles) {
  const filePath = join(eventsPath, file);
  const event = await import(filePath);
  if (event.default.name && event.default.execute) {
    if (event.default.once) {
      client.once(event.default.name, (...args) => event.default.execute(...args));
    } else {
      client.on(event.default.name, (...args) => event.default.execute(...args));
    }
    logger.info(`Loaded event: ${event.default.name}`);
  }
}

client.login(process.env.DISCORD_TOKEN);
```

### src/config.ts
```typescript
export const config = {
  token: process.env.DISCORD_TOKEN || '',
  clientId: process.env.CLIENT_ID || '',
  guildId: process.env.GUILD_ID || '',
  botName: '{{ bot_name }}',
  prefix: '!',
};

if (!config.token) {
  throw new Error('DISCORD_TOKEN is not set');
}

if (!config.clientId) {
  throw new Error('CLIENT_ID is not set');
}
```

### src/utils/logger.ts
```typescript
export const logger = {
  info: (message: string) => console.log(`[INFO] ${new Date().toISOString()} - ${message}`),
  error: (message: string, error?: Error) => {
    console.error(`[ERROR] ${new Date().toISOString()} - ${message}`);
    if (error) console.error(error);
  },
  warn: (message: string) => console.warn(`[WARN] ${new Date().toISOString()} - ${message}`),
  debug: (message: string) => console.debug(`[DEBUG] ${new Date().toISOString()} - ${message}`),
};
```

### src/commands/ping.ts
```typescript
import { SlashCommandBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('ping')
    .setDescription('Replies with pong!'),
  
  async execute(interaction: any) {
    const latency = Math.round(interaction.client.ws.ping);
    await interaction.reply(`🏓 Pong! Latency: ${latency}ms`);
  },
};
```

### src/commands/help.ts
```typescript
import { SlashCommandBuilder, EmbedBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('help')
    .setDescription('Shows available commands'),
  
  async execute(interaction: any) {
    const commands = interaction.client.commands;
    
    const embed = new EmbedBuilder()
      .setColor('#0099ff')
      .setTitle('{{ bot_name }} - Available Commands')
      .setDescription('Here are all available slash commands:');
    
    for (const [name, command] of commands) {
      embed.addFields({
        name: `/${name}`,
        value: command.data.description || 'No description',
        inline: false,
      });
    }
    
    await interaction.reply({ embeds: [embed] });
  },
};
```

### src/commands/user-info.ts
```typescript
import { SlashCommandBuilder, EmbedBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('userinfo')
    .setDescription('Get information about a user')
    .addUserOption(option =>
      option
        .setName('user')
        .setDescription('The user to get info about')
        .setRequired(false)
    ),
  
  async execute(interaction: any) {
    const user = interaction.options.getUser('user') || interaction.user;
    const member = await interaction.guild.members.fetch(user.id);
    
    const embed = new EmbedBuilder()
      .setColor('#0099ff')
      .setTitle(`User Information - ${user.username}`)
      .setThumbnail(user.displayAvatarURL())
      .addFields(
        { name: 'Username', value: user.username, inline: true },
        { name: 'ID', value: user.id, inline: true },
        { name: 'Created', value: user.createdAt.toDateString(), inline: true },
        { name: 'Joined Server', value: member.joinedAt?.toDateString() || 'Unknown', inline: true },
        { name: 'Roles', value: member.roles.cache.map(r => r.name).join(', ') || 'None', inline: false }
      );
    
    await interaction.reply({ embeds: [embed] });
  },
};
```

### src/events/ready.ts
```typescript
import { logger } from '../utils/logger.js';

export default {
  name: 'ready',
  once: true,
  
  execute(client: any) {
    logger.info(`✅ Bot logged in as ${client.user.tag}`);
    client.user.setActivity('{{ bot_name }}', { type: 'WATCHING' });
  },
};
```

### src/events/interactionCreate.ts
```typescript
import { logger } from '../utils/logger.js';

export default {
  name: 'interactionCreate',
  
  async execute(interaction: any) {
    if (!interaction.isChatInputCommand()) return;
    
    const command = interaction.client.commands.get(interaction.commandName);
    
    if (!command) {
      logger.error(`No command matching ${interaction.commandName} was found.`);
      return;
    }
    
    try {
      await command.execute(interaction);
    } catch (error) {
      logger.error(`Error executing ${interaction.commandName}`, error as Error);
      await interaction.reply({
        content: 'There was an error while executing this command!',
        ephemeral: true,
      });
    }
  },
};
```

### src/events/messageCreate.ts
```typescript
import { logger } from '../utils/logger.js';

export default {
  name: 'messageCreate',
  
  async execute(message: any) {
    if (message.author.bot) return;
    
    logger.debug(`Message from ${message.author.tag}: ${message.content}`);
  },
};
```

### Dockerfile
```dockerfile
FROM node:22-slim

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY dist ./dist

CMD ["node", "dist/index.js"]
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  bot:
    build: .
    environment:
      DISCORD_TOKEN: {{ DISCORD_TOKEN }}
      CLIENT_ID: {{ CLIENT_ID }}
      GUILD_ID: {{ GUILD_ID }}
    restart: unless-stopped
```

### .env.example
```
DISCORD_TOKEN=your_bot_token_here
CLIENT_ID=your_client_id_here
GUILD_ID=your_guild_id_here
```

## Getting Started

1. Create a Discord application at [Discord Developer Portal](https://discord.com/developers/applications)

2. Install dependencies:
   ```bash
   npm install
   ```

3. Set up environment variables:
   ```bash
   cp .env.example .env
   # Edit .env with your Discord credentials
   ```

4. Build TypeScript:
   ```bash
   npm run build
   ```

5. Run the bot:
   ```bash
   npm start
   ```

6. Or run in development mode:
   ```bash
   npm run dev
   ```

## Registering Slash Commands

To register slash commands with Discord:

```typescript
import { REST, Routes } from 'discord.js';

const rest = new REST().setToken(process.env.DISCORD_TOKEN);

await rest.put(
  Routes.applicationGuildCommands(clientId, guildId),
  { body: commands }
);
```

## Features

- ✅ Slash commands support
- ✅ Event handlers
- ✅ TypeScript support
- ✅ Docker ready
- ✅ Modular command structure
- ✅ Logging system
