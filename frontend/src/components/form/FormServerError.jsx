import { Typography } from "@mui/material";
import { useFormState } from "react-hook-form";

const FormServerError = (props) => {
  const { errors } = useFormState();

  return errors?.root?.serverError ? <Typography color="error" {...props}>{errors.root.serverError.message}</Typography> : null;
};

export default FormServerError;