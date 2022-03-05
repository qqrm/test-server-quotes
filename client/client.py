from asyncore import loop
import json
import asyncio
from unicodedata import name
import aiohttp
import hashlib

from attr import has


async def simple_integration_test():
    username = "one"
    password = "pass1"

    # get time
    resp = await aiohttp.ClientSession().request(
        "post", 'http://localhost:9999/time',
        data=json.dumps({"login": username}),
        headers={"content-type": "application/json"})
    print(str(resp))
    print(await resp.text())

    assert 200 == resp.status

    json_resp = json.loads(await resp.json())

    assert json_resp["time"] > 0

    time = str(json_resp["time"])
    print("time: ", time)

    # login

    hash: str = hashlib.md5(str(time + password).encode('utf-8')).hexdigest()
    print("hash: ", hash)

    resp = await aiohttp.ClientSession().request(
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

    print(str(resp))
    print(await resp.text())

    # get quote

    hash: str = json_resp["hash"]
    print("new hash: ", hash)

    difficult = int(json_resp["difficulty"])
    pow = 0
    new_hash = "    "
    data = ""

    while new_hash[:4] != str("0"*difficult):
        pow += 1
        data = str(hash + str(pow))
        new_hash: str = hashlib.md5(
            data.encode('utf-8')).hexdigest()

    print("hash + pow ", data)
    print("new hash: {}", new_hash)

    resp = await aiohttp.ClientSession().request(
        "post", 'http://localhost:9999/quote',
        data=json.dumps({
            "login": username,
            "pow": pow
        }),
        headers={"content-type": "application/json"})

    assert 200 == resp.status
    json_resp = json.loads(await resp.json())

    print(str(resp))
    print(await resp.text())

    # get one more quote

    hash: str = json_resp["hash"]
    print("new hash: ", hash)

    difficult = int(json_resp["difficulty"])
    pow = 0
    new_hash = "    "
    data = ""

    while new_hash[:4] != str("0"*difficult):
        pow += 1
        data = str(hash + str(pow))
        new_hash: str = hashlib.md5(
            data.encode('utf-8')).hexdigest()

    print("hash + pow ", data)
    print("new hash: {}", new_hash)

    resp = await aiohttp.ClientSession().request(
        "post", 'http://localhost:9999/quote',
        data=json.dumps({
            "login": username,
            "pow": pow
        }),
        headers={"content-type": "application/json"})
    print(str(resp))
    print(await resp.text())


asyncio.get_event_loop().run_until_complete(simple_integration_test())
