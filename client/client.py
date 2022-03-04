import json
import asyncio
from unicodedata import name
import aiohttp
import hashlib


async def simple_integration_test():
    username = "one"
    password = "pass1"

    # get time
    resp = await aiohttp.ClientSession().request(
        "post", 'http://localhost:9999/gettime',
        data=json.dumps({"login": username}),
        headers={"content-type": "application/json"})
    print(str(resp))
    print(await resp.text())

    assert 200 == resp.status

    json_resp = json.loads(await resp.json())

    assert len(json_resp["time"]) > 0

    time: str = json_resp["time"]
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
    print(str(resp))
    print(await resp.text())


asyncio.get_event_loop().run_until_complete(simple_integration_test())
