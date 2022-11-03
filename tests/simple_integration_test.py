from asyncore import loop
import json
import asyncio
from unicodedata import name
import aiohttp
import hashlib

from attr import has


# mb still looks bad, but works 🙄

async def get_time(session, username, password):
    resp = await session.request(
        "post", 'http://localhost:9999/time',
        data=json.dumps({"login": username}),
        headers={"content-type": "application/json"})

    assert 200 == resp.status
    json_resp = json.loads(await resp.json())

    assert json_resp["time"] > 0
    time = str(json_resp["time"])

    return time


async def login(session, time, username, password):
    hash: str = hashlib.md5(str(time + password).encode('utf-8')).hexdigest()

    resp = await session.request(
        "post", 'http://localhost:9999/auth',
        data=json.dumps({
            "login": username,
            "hash": hash
        }),
        headers={"content-type": "application/json"})

    assert 200 == resp.status
    json_resp = json.loads(await resp.json())
    assert str(json_resp["hash"]) == hashlib.md5(
        hash.encode('utf-8')).hexdigest()

    hash: str = json_resp["hash"]
    difficult = json_resp["difficulty"]
    return hash, difficult


async def logout(session, hash, username, password):

    data = str(hash + password)

    new_hash: str = hashlib.md5(
        data.encode('utf-8')).hexdigest()

    resp = await session.request(
        "post", 'http://localhost:9999/logout',
        data=json.dumps({
            "login": username,
            "hash": new_hash
        }),
        headers={"content-type": "application/json"})

    assert 200 == resp.status

    await session.close()


def calc_pow(difficult, hash):
    pow = 0
    new_hash = "    "
    data = ""

    while new_hash[:4] != str("0"*difficult):
        pow += 1
        data = str(hash + str(pow))
        new_hash: str = hashlib.md5(
            data.encode('utf-8')).hexdigest()

    return pow


async def simple_integration_test(username="one", password="pass1"):
    session = aiohttp.ClientSession()
    time = await get_time(session, username, password)
    hash, difficult = await login(session, time, username, password)

    # get quote
    pow = calc_pow(difficult, hash)
    resp = await session.request(
        "post", 'http://localhost:9999/quote',
        data=json.dumps({
            "login": username,
            "pow": pow
        }),
        headers={"content-type": "application/json"})

    assert 200 == resp.status
    json_resp = json.loads(await resp.json())
    # print(json_resp["quote"])

    # get one more quote
    hash: str = json_resp["hash"]

    difficult = int(json_resp["difficulty"])
    pow = calc_pow(difficult, hash)
    resp = await session.request(
        "post", 'http://localhost:9999/quote',
        data=json.dumps({
            "login": username,
            "pow": pow
        }),
        headers={"content-type": "application/json"})

    assert 200 == resp.status

    json_resp = json.loads(await resp.json())
    # print(json_resp["quote"])

    # logout
    hash: str = json_resp["hash"]
    await logout(session, hash, username, password)
    await session.close()

    print("# SUCCESS for", username)


async def simple_multiclient_test():
    users = [["one", "pass1"], ["two", "pass2"], ["three", "pass3"]]

    for user in users:
        await simple_integration_test(user[0], user[1])

    print("# SUCCESS MULTI")


async def tests():
    await simple_integration_test()
    await simple_multiclient_test()


asyncio.get_event_loop().run_until_complete(tests())
