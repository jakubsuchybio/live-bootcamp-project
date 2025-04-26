  @ECHO OFF

  REM Define the location of the .env file (relative path)
  SET ENV_FILE=auth-service\.env

  REM Check if the .env file exists
  IF EXIST "%ENV_FILE%" (
    ECHO Found .env file.
  ) ELSE (
    ECHO Error: .env file not found!
    EXIT /B 1
  )

  FOR /F "tokens=*" %%a IN (%ENV_FILE%) DO (
    SET %%a
  )

  REM Run docker-compose commands with loaded variables
  docker-compose build
  docker-compose up

  ECHO Finished!
