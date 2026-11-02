# PatientCare

A Windows Forms desktop app for a small veterinary clinic. It keeps track of pet owners, their animals (patients) and daily to-dos in a local SQLite database.

## Features

- Add and edit pet owners (clients) and the animals they own
- Patient records: name, gender, registration date and notes
- Operations screen as the main entry point, plus to-do and checklist screens
- Local SQLite database, created on first run
- Dependency injection with `Microsoft.Extensions.DependencyInjection`
- Global exception handling
- Modern UI controls from [ReaLTaiizor](https://github.com/Taiizor/ReaLTaiizor)

## Tech stack

C# · .NET 9 (`net9.0-windows`) · Windows Forms · System.Data.SQLite · Microsoft.Extensions.Configuration

## Getting started

Requirements: Windows and the .NET 9 SDK (or Visual Studio 2022 17.12+).

```bash
git clone https://github.com/muhammedalikutbay/PatientsCare.git
cd PatientsCare
dotnet run --project PatientCare
```

Or open `PatientCare.sln` in Visual Studio and press **F5**.

The connection string lives in `PatientCare/appsettings.json`. By default the app uses `HastaTakipDB.db` next to the executable.

## Project structure

```text
PatientCare/
├── Forms/          UI screens (owners, patients, operations, to-do, settings)
├── Models/         Patient, PatientOwner, Treatment
├── Interfaces/     Repository and database service contracts
├── Repositories/   SQLite data access
├── Services/       Database creation
└── Handlers/       Global exception handling
```

## Status

Early version (v0.01 pre-release). The UI is in Turkish. Some screens, such as treatments and the balance sheet, are placeholders.

## License

[MIT](LICENSE)
