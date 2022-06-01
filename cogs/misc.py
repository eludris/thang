import nextcord
from nextcord.ext import commands


class Misc(commands.Cog):
    """A miscellaneous class for miscellaneous things."""

    def __init__(self, client):
        self.client = client

    @commands.command(
        brief="Ping Pong Ding Dong",
        description="The generic must have ping command."
    )
    async def ping(self, ctx):
        await ctx.reply("Pong!")


def setup(client):
    client.add_cog(Misc(client))
