#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = ["boto3", "awscrt", "python-dotenv", "nanoid"]
# ///
"""
Seed the <db_prefix>_test_pagination DynamoDB table for the pagination
end-to-end integration test.

Empties the table, then writes exactly 62 rows:
  id:       4-character nanoid (unique)
  group_id: 1 (number, same for every row)
  number:   1..62 (number)
  name:     a single char from 0-9a-zA-Z, one per row
  odd:      number 1 when `number` is odd, absent otherwise
  even:     string "y" when `number` is even, absent otherwise
  mod5:     number % 5

Reads DB_PREFIX from ../.env. Uses the `seslogin` AWS profile in ap-southeast-2
by default (override with --profile), matching the Terraform provider in
infra/main.tf.
"""

import argparse
import os
import sys
from pathlib import Path

import boto3
from dotenv import load_dotenv
from nanoid import generate

REGION = "ap-southeast-2"
PROFILE = "seslogin"
ALPHABET = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
ID_SIZE = 4

load_dotenv(Path(__file__).parent.parent / ".env")
DB_PREFIX = os.environ.get("DB_PREFIX")
if not DB_PREFIX:
    print("ERROR: DB_PREFIX not set in .env")
    sys.exit(1)

TABLE_NAME = f"{DB_PREFIX}_test_pagination"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--profile", default=PROFILE, help=f"AWS profile (default {PROFILE})")
    args = ap.parse_args()

    session = boto3.Session(profile_name=args.profile, region_name=REGION)
    table = session.resource("dynamodb").Table(TABLE_NAME)

    # Empty the table.
    deleted = 0
    scan_kwargs = {"ProjectionExpression": "id"}
    with table.batch_writer() as batch:
        while True:
            resp = table.scan(**scan_kwargs)
            for item in resp["Items"]:
                batch.delete_item(Key={"id": item["id"]})
                deleted += 1
            if "LastEvaluatedKey" not in resp:
                break
            scan_kwargs["ExclusiveStartKey"] = resp["LastEvaluatedKey"]
    print(f"Deleted {deleted} existing row(s) from {TABLE_NAME}.")

    if len(ALPHABET) != 62:
        raise AssertionError(f"expected 62 name chars, got {len(ALPHABET)}")

    # Generate 62 unique 4-char ids.
    ids: set[str] = set()
    while len(ids) < 62:
        ids.add(generate(size=ID_SIZE))
    ids_list = list(ids)

    with table.batch_writer() as batch:
        for n in range(1, 63):
            item = {
                "id": ids_list[n - 1],
                "group_id": 1,
                "number": n,
                "name": ALPHABET[n - 1],
                "mod5": n % 5,
            }
            if n % 2 == 1:
                item["odd"] = 1
            else:
                item["even"] = "y"
            batch.put_item(Item=item)
    print(f"Wrote 62 rows to {TABLE_NAME}.")


if __name__ == "__main__":
    main()
