import { Typography } from "@mui/material";
import { useFormContext } from "react-hook-form";

const FormServerError = (props) => {
  const { formState: { errors } } = useFormContext();

  return errors?.root?.serverError ? <Typography color="error" {...props}>{errors.root.serverError.message}</Typography> : null;
};

export default FormServerError;