"""Create a synthetic PatientsCare database for end-to-end import checks.

Usage: python make-legacy-db.py <output.db>
Uses the exact CREATE TABLE statements of the legacy app (docs/legacy-schema.md) and fake data only,
including the known problems: integer phones without the leading zero, Turkish long birth dates,
dd-MM-yyyy registration dates, an orphan patient and a free-text species.
"""
import sqlite3
import sys
from pathlib import Path

out = Path(sys.argv[1])
if out.exists():
    out.unlink()
db = sqlite3.connect(out)
db.executescript(
    """
    CREATE TABLE PatientOwner("Id" INTEGER, "OwnerName" TEXT, "OwnerAdress" TEXT, "OwnerPhone" INTEGER, PRIMARY KEY("Id"));
    CREATE TABLE Patient("Id" INTEGER, "OwnerId" INTEGER, "PatientName" TEXT, "PatientType" TEXT, "PatientGender" TEXT,
      "BirthDate" TEXT, "RegistrationDate" TEXT, "PatientNote" TEXT, PRIMARY KEY("Id"),
      FOREIGN KEY("OwnerId") REFERENCES "PatientOwner"("Id"));
    CREATE TABLE ToDo("Id" INTEGER, "OwnerId" INTEGER, "PatientId" INTEGER, "ToDoName" TEXT, "ToDoDate" DATETIME,
      "ToDoNote" TEXT, PRIMARY KEY("Id"), FOREIGN KEY("OwnerId") REFERENCES "PatientOwner"("Id"),
      FOREIGN KEY("PatientId") REFERENCES "Patient"("Id"));
    """
)
db.executemany(
    "INSERT INTO PatientOwner VALUES (?, ?, ?, ?)",
    [
        (1, "Deneme Yılmaz", "Örnek Mah. 1", 5551112233),
        (2, "Sınama Kaya", "Örnek Mah. 2", "0555 444 55 66"),
    ],
)
db.executemany(
    "INSERT INTO Patient VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    [
        (1, 1, "Tarçın", "Kedi", "Dişi", "4 Kasım 2024 Pazartesi", "02-01-2025", "Örnek not"),
        (2, 1, "Zeytin", "Köpek", "Erkek", "12 Mart 2023 Pazar", "03-01-2025", None),
        (3, 2, "Limon", "Muhabbet kuşu", "Erkek", "", "05-01-2025", None),
        (4, 99, "Sahipsiz Örnek", "Kedi", "Dişi", "", "06-01-2025", None),
    ],
)
db.executemany(
    "INSERT INTO ToDo VALUES (?, ?, ?, ?, ?, ?)",
    [
        (1, 1, 1, "Karma aşı 2. doz", "2025-02-01", None),
        (2, 1, 2, "Kuduz aşısı", "2030-01-15", "Örnek gelecek tarih"),
    ],
)
db.commit()
db.close()
print(out)
