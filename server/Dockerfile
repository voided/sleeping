FROM mcr.microsoft.com/dotnet/sdk:8.0-jammy AS build
WORKDIR /app

COPY . ./
# Restore as distinct layers

RUN dotnet restore

# Build and publish a release
RUN dotnet publish -c Release -o out


# Build runtime image
FROM mcr.microsoft.com/dotnet/aspnet:8.0-jammy-chiseled AS runtime
WORKDIR /app

# copy published app to runtime /app folder
COPY --from=build /app/out .

# aspnet apps listen on port 8080 by default
EXPOSE 8080

ENTRYPOINT ["dotnet", "Server.dll"]
