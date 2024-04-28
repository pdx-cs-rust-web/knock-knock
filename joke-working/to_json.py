#!/usr/bin/env python3
import argparse, json, re, sys
ap = argparse.ArgumentParser()
ap.add_argument(
    "--source",
    default="https://lajollamom.com/kid-friendly-knock-knock-jokes",
)
ap.add_argument("jokefile")
args = ap.parse_args()

CATEGORY_LINE = re.compile(r"^tag: (.*)$")
START_LINE = re.compile(r"^[0-9]+\. Knock, knock\.$")
WHO = re.compile(r"^(.*)[!.]$")
LAST_WORD = re.compile(r".* ([A-za-z]+)[^A-Za-z]*$")

def make_tag(who):
    return who.strip().lower().replace(" ", "-")

jokes = dict()

def amend_tag(tag):
    global jokes
    joke = jokes[tag]
    answer = joke["answer_who"]
    match = LAST_WORD.match(answer)
    if match is None:
        print("bad answer:", answer, file=sys.stderr)
        return
    last_word = make_tag(match[1])
    del jokes[tag]
    new_tag = tag + "-" + last_word
    joke["id"] = new_tag
    jokes[new_tag] = joke

category = "misc"

lines = open(args.jokefile, "r")
try:
    while True:
        line = next(lines)
        match = CATEGORY_LINE.match(line)
        if match is not None:
            category = match[1]
            continue
        if START_LINE.match(line) is None:
            continue
        #print("started", file=sys.stderr)
        line = next(lines).strip()
        if line != "Who’s there?":
            continue
        #print("whos", file=sys.stderr)
        line = next(lines).strip()
        who = WHO.match(line)
        if who is None:
            continue

        #print("who", file=sys.stderr)
        who = who[1]
        question = next(lines)
        answer = next(lines).strip()
        blank = next(lines).strip()
        if blank != "":
            print(f"bad format: {tag}: {blank}", file=sys.stderr)
            continue
        #print("blank", file=sys.stderr)
        tag = make_tag(who)

        categories = ["kids"]
        if category != "misc":
            categories.append(category)
        joke = {
            "id" : tag,
            "whos_there" : who,
            "answer_who" : answer,
            "tags" : categories,
        }
        if args.source:
            joke["source"] = args.source

        if tag in jokes:
            amend_tag(tag)
            jokes[tag] = joke
            amend_tag(tag)
        else:
            jokes[tag] = joke

except StopIteration as _:
    pass

for tag, joke in jokes.items():
    #print("tag", file=sys.stderr)
    filename = "jokes/" + tag + ".json"
    with open(filename, "w") as f:
        json.dump(joke, f)
