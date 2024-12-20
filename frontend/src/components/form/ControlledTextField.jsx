import React from "react";
import { Controller, useWatch } from "react-hook-form";
import { OutlinedInput, FormControl, FormLabel, IconButton, InputAdornment, FormHelperText } from "@mui/material";

import Visibility from '@mui/icons-material/Visibility';
import VisibilityOff from '@mui/icons-material/VisibilityOff';

const ControlledTextField = ({
  name,
  helperText,
  fullWidth,
  required = false,
  label = "",
  type = "text",
  ...props
}) => {
  const { control } = useWatch(name);
  const [showPassword, setShowPassword] = React.useState(false);
  const handleClickShowPassword = () => setShowPassword((show) => !show);

  const handleMouseDownPassword = (event) => {
    event.preventDefault();
  };

  const handleMouseUpPassword = (event) => {
    event.preventDefault();
  };

  const endAdornment = type == 'password' && (
    <InputAdornment position="end">
      <IconButton
        aria-label={
          showPassword ? 'hide the password' : 'display the password'
        }
        onClick={handleClickShowPassword}
        onMouseDown={handleMouseDownPassword}
        onMouseUp={handleMouseUpPassword}
        edge="end"
        tabIndex={-1}
      >
        {showPassword ? <VisibilityOff /> : <Visibility />}
      </IconButton>
    </InputAdornment>
  );

  return (
    <Controller
      name={name}
      control={control}
      render={({ field, fieldState }) => (
        <FormControl fullWidth={fullWidth}>
          <FormLabel required={required} sx={{ mb: 1 }}>{label}</FormLabel>
          <OutlinedInput
            {...field}
            type={type == 'password' ? (showPassword ? 'text' : 'password') : type}
            error={!!fieldState.error}
            endAdornment={endAdornment}
            {...props}
          />
          <FormHelperText error={!!fieldState.error}>
            {fieldState.error?.message || helperText}
          </FormHelperText>
        </FormControl>
      )}
    />
  );
};

export default ControlledTextField;
