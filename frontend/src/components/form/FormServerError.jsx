import { Alert } from "@mui/material";
import { useFormState } from "react-hook-form";

const FormServerError = (props) => {
  const { errors } = useFormState();

  return errors?.root?.serverError ? <Alert severity="error" {...props}>{errors.root.serverError.message}</Alert> : null;
};

export default FormServerError;