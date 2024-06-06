import json, random
from urllib.request import Request, urlopen
from urllib.error import HTTPError

url = "http://localhost:3000/api/v1"

token = None

def curl(endpoint, data=None, method="GET", expect_error=False, use_token = True):
    headers = {"Content-Type":  "application/json"}
    if use_token:
        headers["Authorization"] = "Bearer " + token
    if data:
        data = bytes(json.dumps(data), encoding="utf-8")
    req = Request(
        url + endpoint,
        headers = headers,
        data = data,
        method = method,
    )
    try:
        response = urlopen(req).read()
        if response:
            response = json.loads(response)
        if not expect_error:
            return response
        assert False
    except HTTPError as e:
        if expect_error:
            r = json.loads(e.read())
            return (e, r["error"])
        assert False

password = open("db/reg-password.txt", "r").read().strip()
reg = {
    "full_name": "Bart Massey",
    "email": "bart.massey@gmail.com",
    "password": password,
}

print("registering: ", end="")
token_data = curl(
    "/register",
    data = reg,
    use_token = False,
)
token = token_data["access_token"]
print("registered")

reg["password"] = ""

print("testing for registration failure: ", end="")
e, _ = curl(
    "/register",
    data = reg,
    expect_error = True,
    use_token = False,
)
assert str(e) == "HTTP Error 401: Unauthorized"
print("failed successfully")

joke = {
  "answer_who": "You don't have to cry about it!",
  "id": "boo",
  "source": "http://example.com/knock-knock-jokes",
  "tags": [
    "kids",
    "food"
  ],
  "whos_there": "Boo"
}


print("adding existing joke: ", end="")
e, r = curl(
    "/joke/add",
    method = "POST",
    data = joke,
    expect_error = True,
)
assert str(e) == "HTTP Error 400: Bad Request"
assert r == {"JokeExists": "boo"}
print("failed successfully")

def get_random_number():
    return random.randrange(900_000) + 1000
random_number = get_random_number()

joke_id = f"random-number-{random_number}"
joke = {
  "whos_there": "Random Number",
  "answer_who": f"Random Number {random_number}",
  "id": joke_id,
  "source": "Test joke, please ignore",
  "tags": [
    "deleteme",
  ],
}

try:
    print("adding new joke without auth: ", end="")
    e, _ = curl(
        "/joke/add",
        method = "POST",
        data = joke,
        expect_error = True,
        use_token = False,
    )
    assert str(e) == "HTTP Error 401: Unauthorized"
    print("failed successfully")

    print("adding new joke: ", end="")
    curl(
        "/joke/add",
        method = "POST",
        data = joke,
    )
    print("ok")

    joke["answer_who"] = f"Random Number {get_random_number()}"
    print("updating new joke: ", end="")
    curl(
        f"/joke/{joke_id}",
        method = "PUT",
        data = joke,
    )
    print("ok")
finally:
    print("deleting new joke: ", end="")
    curl(
        f"/joke/{joke_id}",
        method = "DELETE",
    )
    print("ok")
