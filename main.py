from nextcord.ext import commands
import nextcord
from dotenv import load_dotenv
from os import getenv, listdir


bot = commands.Bot(command_prefix="-")
GUILDS = [980412957060137001]


@bot.event
async def on_ready():
    print(f"Logged in as {bot.user}!")

bot.load_extension("jishaku")

for cog in listdir("cogs"):
    if cog.endswith(".py"):
        bot.load_extension(f"cogs.{cog[:-3]}")

load_dotenv()

if token := getenv("TOKEN"):
    bot.run(token)
else:
    print("TOKEN env variable not found.")
