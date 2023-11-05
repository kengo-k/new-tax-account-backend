#!/bin/bash
sqlite3 -separator , database.db ".import testdata.csv posts"
