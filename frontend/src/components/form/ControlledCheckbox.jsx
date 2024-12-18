import { Controller, useWatch } from "react-hook-form";
import { Checkbox, FormControlLabel } from "@mui/material";

const ControlledCheckbox = ({ name, label, disabled = false }) => {
  const { control } = useWatch(name);

  return (
    <Controller
      name={name}
      control={control}
      render={({ field }) => (
        <FormControlLabel
          label={label}
          control={
            <Checkbox
              color="primary"
              checked={Boolean(field.value)}
              disabled={disabled}
              onChange={(event, checked) => field.onChange(checked)}
            />
          }
        />
      )}
    />
  );
};

export default ControlledCheckbox;
